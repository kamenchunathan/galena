use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::{self, Child, Command},
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use anyhow::{self, Context, Result};
use clap::{Parser, Subcommand};
use include_dir::{include_dir, Dir};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{debug, error, info, warn, Level};

static FRONTEND_DIR: Dir = include_dir!("$FRONTEND_DIST_DIR");
static WASM_BINDGEN_EXPORTS: &'static str = include_str!(env!("WASM_BINDGEN_EXPORTS"));

const GALENA_DIR: &str = ".galena";
const DEBOUNCE_MS: u64 = 100;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let cli = Cli::parse();

    let build_dir = Path::new(GALENA_DIR).join("build");
    let dist_dir = Path::new(GALENA_DIR).join("dist");
    let roc_bin = &cli.roc_bin.unwrap_or("roc".to_string());

    // Check if roc binary exists and is executable
    match process::Command::new(roc_bin).arg("--version").status() {
        Ok(status) => {
            if !status.success() {
                warn!("Error running roc command");
            }
        }
        Err(e) => return Err(format!("Roc binary '{}' not found: {}", roc_bin, e).into()),
    }

    match cli.action {
        Action::Build { input } => {
            let input = Path::new(&input);
            execute_build(roc_bin, &build_dir, &dist_dir, &input)?;
        }

        Action::Run { input } => {
            let input = Path::new(&input);
            execute_build(roc_bin, &build_dir, &dist_dir, &input)?;
            execute_run(&build_dir, &dist_dir, &input)?;
        }

        Action::Watch { input, paths } => {
            // Watch for file changes
            let input = Path::new(&input);
            watch_files(roc_bin, &build_dir, &dist_dir, &input, paths)?;
        }
    }

    Ok(())
}

fn execute_build(roc_bin: &str, build_dir: &Path, dist_dir: &Path, input: &Path) -> Result<()> {
    // Validate input file exists
    if let Err(e) = fs::metadata(input) {
        return Err(anyhow::anyhow!(
            "Input file '{}' not found: {}",
            input.display(),
            e
        ));
    }

    // Create dist directory
    create_directory_if_not_exists(dist_dir)?;

    // Build WASM
    build_wasm(roc_bin, build_dir, input)?;

    // Copy WASM bundle all frontend assets to dist directory
    copy_frontend_to_dist(dist_dir)?;
    copy_wasm_to_dist(build_dir, dist_dir, input)?;

    info!(
        "WASM build completed. Files written to {}.",
        dist_dir.to_str().unwrap()
    );

    // Build backend binary
    info!("Building backend");
    let input_file_name = Path::new(input).file_stem().unwrap().to_str().unwrap();
    let output_binary = build_dir.join(input_file_name);
    build_backend(roc_bin, build_dir, input, &output_binary)?;

    Ok(())
}

fn execute_run(build_dir: &Path, dist_dir: &Path, input: &Path) -> Result<Child> {
    create_directory_if_not_exists(build_dir)?;

    let input_file_name = Path::new(input).file_stem().unwrap().to_str().unwrap();
    let output_binary = build_dir.join(input_file_name);

    // Run the backend binary
    // Get absolute path to dist_dir for environment variable
    let dist_dir_abs = fs::canonicalize(dist_dir).context(format!(
        "Failed to get absolute path to {}",
        dist_dir.display()
    ))?;

    let mut run_cmd = Command::new(&output_binary);
    run_cmd.env("DIST_DIR", dist_dir_abs.to_str().unwrap());

    info!("Running backend with DIST_DIR={}", dist_dir_abs.display());
    let child = run_cmd.spawn().context(format!(
        "Failed to execute backend binary {}",
        &output_binary.display()
    ))?;

    Ok(child)
}

fn watch_files(
    roc_bin: &str,
    build_dir: &Path,
    dist_dir: &Path,
    input: &Path,
    additional_paths: Vec<String>,
) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    let input_path = Path::new(input);
    info!("Watching : {}", input_path.display());
    watcher.watch(&input_path, RecursiveMode::Recursive)?;

    // Watch additional paths if specified
    for path in &additional_paths {
        let path = Path::new(path);
        if path.exists() {
            info!("Watching additional path: {}", path.display());
            watcher.watch(path, RecursiveMode::Recursive)?;
        } else {
            warn!("Skipping non-existent path: {}", path.display());
        }
    }

    let mut backend_proc: Option<Child> = None;
    let mut last_rebuild = Instant::now();

    let roc_bin_clone = roc_bin.to_string();
    let build_dir_clone = build_dir.to_path_buf();
    let dist_dir_clone = dist_dir.to_path_buf();

    info!("Building app");
    if let Err(e) = execute_build(&roc_bin_clone, &build_dir_clone, &dist_dir_clone, &input) {
        error!("Build error: {}", e);
    } else {
        info!("Running backend");
        backend_proc = execute_run(&build_dir_clone, &dist_dir_clone, &input).ok();
    }

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                let now = Instant::now();
                if now.duration_since(last_rebuild) > Duration::from_millis(DEBOUNCE_MS) {
                    debug!("File change detected: {:?}", event.paths);

                    info!("Stopping build thread");
                    if let Some(mut proc) = backend_proc.take() {
                        let _ = proc.kill();
                    }

                    info!("Building app");
                    if let Err(e) =
                        execute_build(&roc_bin_clone, &build_dir_clone, &dist_dir_clone, &input)
                    {
                        error!("Build error: {}", e);
                        continue;
                    } else {
                        info!("Running backend");
                        backend_proc = execute_run(&build_dir_clone, &dist_dir_clone, &input).ok();
                    }

                    last_rebuild = Instant::now();
                }
            }
            Ok(Err(e)) => error!("Watch error: {}", e),
            Err(e) => {
                error!("Watch channel error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn build_wasm(roc_bin: &str, build_dir: &Path, input: &Path) -> Result<()> {
    create_directory_if_not_exists(build_dir)?;

    let wasm_obj_path = build_dir
        .join(input.file_stem().context("Could not get input file stem")?)
        .with_extension("o");

    debug!("Building WASM object file");
    let status = build_wasm_obj_cmd(
        roc_bin,
        wasm_obj_path.to_str().unwrap(),
        input.to_str().unwrap(),
    )?
    .status()
    .context("Unable to spawn roc build command")?;

    if !status.success() {
        // Exit code 2 is treated as a warning
        if status.code() == Some(2) {
            warn!("WASM build completed with warnings (status: {})", status);
            return Ok(());
        } else {
            return Err(anyhow::anyhow!("WASM build failed with status: {}", status));
        }
    }

    // Linking
    let libfrontend_host_path = build_dir.join("libfrontend_host.a");
    let mut libfrontend_host = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&libfrontend_host_path)?;

    let frontend_host_archive: Vec<_> =
        include_bytes!("../../../target/wasm32-unknown-unknown/release/libfrontend_host.a")
            .to_vec();
    libfrontend_host
        .write_all(&frontend_host_archive)
        .context("Could not write frontend host archive to temporary file")?;

    let link_output_path = build_dir
        .join(input.file_stem().context("Could not get input file stem")?)
        .with_extension("wasm");
    let exports = WASM_BINDGEN_EXPORTS
        .split("\n")
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let status = link_wasm_cmd(
        dbg!(&exports),
        libfrontend_host_path.to_str().unwrap(),
        wasm_obj_path.to_str().unwrap(),
        link_output_path.to_str().unwrap(),
    )?
    .status()
    .context("Could not run WASM linking command")?;

    match status.code() {
        Some(0) => {
            info!("Successfully completed linking")
        }
        _ => {
            return Err(anyhow::anyhow!(
                "WASM linking failed with status {}",
                status
            ))
        }
    }

    // Wasm bindgen
    wasm_bindgen_cmd(
        build_dir
            .to_str()
            .context("Convert build-dir path to str")?,
        link_output_path
            .to_str()
            .context("Convert link output path to str")?,
    )?
    .status()?;

    Ok(())
}

fn copy_wasm_to_dist(build_dir: &Path, dist_dir: &Path, input: &Path) -> Result<()> {
    debug!("Copying wasm file to dist");
    let input_file_name = format!(
        "{}_bg.wasm",
        input
            .file_stem()
            .context("Could not get file stem")?
            .to_str()
            .context("Could not convert filename to string")?
    );
    let source_wasm = build_dir.join(input_file_name);
    let dest_wasm = dist_dir.join("app.wasm");

    fs::copy(&source_wasm, &dest_wasm).context(format!(
        "Failed to copy WASM from {} to {}",
        source_wasm.to_str().unwrap(),
        dest_wasm.to_str().unwrap()
    ))?;

    Ok(())
}

fn build_backend(
    roc_bin: &str,
    build_dir: &Path,
    input: &Path,
    output_binary: &Path,
) -> Result<()> {
    let status = build_backend_cmd(
        roc_bin,
        build_dir.to_str().unwrap(),
        input.to_str().unwrap(),
        output_binary.to_str().unwrap(),
    )?
    .status()
    .context("Unable to spawn roc build command for backend")?;

    if !status.success() {
        // Exit code 2 is treated as a warning
        if status.code() == Some(2) {
            warn!("Backend build completed with warnings (status: {})", status);
            return Ok(());
        } else {
            return Err(anyhow::anyhow!(
                "Backend build failed with status: {}",
                status
            ));
        }
    }

    Ok(())
}

/// Recursively copy frontend assets to the dist directory while preserving structure
fn copy_frontend_to_dist(dist_dir: &Path) -> Result<()> {
    // Clear the dist directory first to prevent stale files
    if dist_dir.exists() {
        fs::remove_dir_all(dist_dir)?;
        fs::create_dir_all(dist_dir)?;
    }

    for entry in FRONTEND_DIR.entries() {
        let dest_path = dist_dir.join(entry.path());

        match entry {
            include_dir::DirEntry::Dir(dir) => {
                let dir_path = dist_dir.join(dir.path());
                fs::create_dir_all(&dir_path).context(format!(
                    "Failed to create directory: {}",
                    dir_path.display()
                ))?;

                // Recursively handle subdirectories
                for subentry in dir.entries() {
                    copy_dir_entry(subentry, &dist_dir)?;
                }
            }
            include_dir::DirEntry::File(file) => {
                let file_contents = file.contents();
                fs::write(&dest_path, file_contents)
                    .context(format!("Failed to write file: {}", dest_path.display()))?;
            }
        }
    }

    Ok(())
}

fn copy_dir_entry(entry: &include_dir::DirEntry, dist_dir: &Path) -> Result<()> {
    match entry {
        include_dir::DirEntry::Dir(dir) => {
            fs::create_dir_all(dir.path())?;

            for subentry in dir.entries() {
                copy_dir_entry(subentry, dist_dir)?;
            }
        }
        include_dir::DirEntry::File(file) => {
            fs::write(dist_dir.join(file.path()), file.contents())
                .context(format!("Unable to write to file {}", file.path().display()))?;
        }
    }

    Ok(())
}

/// Galena CLI tool
/// Galena is a lamdera clone for roc
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Path to the roc binary
    #[arg(long)]
    roc_bin: Option<String>,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Builds input and prepares frontend assets
    #[command(alias = "b")]
    Build {
        /// Input Roc file
        input: String,
    },

    /// Runs the backend binary with the DIST_DIR environment variable
    #[command(alias = "r")]
    Run {
        /// Input Roc file
        input: String,
    },

    /// Watches for file changes and rebuilds automatically
    #[command(alias = "w")]
    Watch {
        /// Input Roc file
        input: String,

        /// Additional paths to watch (optional)
        #[arg(short, long)]
        paths: Vec<String>,
    },
}

fn create_directory_if_not_exists(dir: &Path) -> Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

fn build_backend_cmd(
    roc_bin_path: &str,
    build_dir: &str,
    source_file: &str,
    output_binary: &str,
) -> anyhow::Result<Command> {
    // NOTE: Not using the roc_bin_path for building the backend as it is only a workaround for
    // building with wasm where wasi-libc is required but not packaged when building with nix

    // We already validate file existence in execute_build
    create_directory_if_not_exists(Path::new(build_dir))?;

    let mut cmd = process::Command::new(roc_bin_path);
    cmd.args([
        "build",
        "--emit-llvm-ir",
        source_file,
        "--output",
        output_binary,
    ]);

    Ok(cmd)
}

fn build_wasm_obj_cmd(
    roc_bin_path: &str,
    output_path: &str,
    source_file: &str,
) -> anyhow::Result<Command> {
    let mut cmd = process::Command::new(roc_bin_path);

    // We're handling linking ourselves
    cmd.args([
        "build",
        "--target",
        "wasm32",
        "--no-link",
        "--output",
        output_path,
        source_file,
    ]);

    Ok(cmd)
}

fn link_wasm_cmd(
    exports: &[&str],
    lib_frontend_path: &str,
    wasm_obj_path: &str,
    output_path: &str,
) -> anyhow::Result<Command> {
    let mut cmd = process::Command::new("wasm-ld");
    let mut args = exports
        .iter()
        .map(|s| format!("--export={s}"))
        .collect::<Vec<_>>();
    args.extend([
        String::from("--no-entry"),
        String::from(lib_frontend_path),
        String::from(wasm_obj_path),
        String::from("-o"),
        String::from(output_path),
    ]);
    cmd.args(args);

    Ok(cmd)
}

fn wasm_bindgen_cmd(build_dir: &str, input: &str) -> anyhow::Result<process::Command> {
    let mut cmd = process::Command::new("wasm-bindgen");

    cmd.args(["--target", "web", "--out-dir", build_dir, input]);

    Ok(cmd)
}
