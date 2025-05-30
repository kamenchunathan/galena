use std::{
    error::Error,
    fs,
    path::Path,
    process::{self, Command},
    sync::mpsc::{channel, Receiver},
    thread,
    time::{Duration, Instant},
};

use anyhow::{self, Context, Result};
use clap::{Parser, Subcommand};
use include_dir::{include_dir, Dir};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{debug, error, info, warn};

static FRONTEND_DIR: Dir = include_dir!("$FRONTEND_DIST_DIR");

const GALENA_DIR: &str = ".galena";
const DEBOUNCE_MS: u64 = 100; // Debounce time in milliseconds

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    let build_dir = Path::new(GALENA_DIR).join("build");
    let dist_dir = Path::new(GALENA_DIR).join("dist");
    let roc_bin = &cli.roc_bin.unwrap_or("roc".to_string());

    // Check if roc binary exists and is executable
    if let Err(e) = fs::metadata(roc_bin) {
        return Err(format!("Roc binary '{}' not found: {}", roc_bin, e).into());
    }

    match cli.action {
        Action::Build { input } => {
            execute_build(roc_bin, &build_dir, &dist_dir, &input)?;
        }
        Action::Run { input } => {
            // First build the app (including WASM), since we need the dist directory
            execute_build(roc_bin, &build_dir, &dist_dir, &input)?;

            // Then build and run the backend
            execute_run(&build_dir, &dist_dir, &input)?;
        }
        Action::Watch { input, paths } => {
            // Watch for file changes
            watch_files(roc_bin, &build_dir, &dist_dir, &input, paths)?;
        }
    }

    Ok(())
}

fn execute_build(roc_bin: &str, build_dir: &Path, dist_dir: &Path, input: &str) -> Result<()> {
    // Validate input file exists
    if let Err(e) = fs::metadata(input) {
        return Err(anyhow::anyhow!("Input file '{}' not found: {}", input, e));
    }

    // Create dist directory and copy all frontend assets
    create_directory_if_not_exists(dist_dir)?;
    copy_frontend_to_dist(dist_dir)?;

    // Build WASM
    build_wasm(roc_bin, build_dir, input)?;

    // Copy WASM bundle to dist directory
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

fn execute_run(build_dir: &Path, dist_dir: &Path, input: &str) -> Result<()> {
    create_directory_if_not_exists(build_dir)?;

    let input_file_name = Path::new(input).file_stem().unwrap().to_str().unwrap();
    let output_binary = build_dir.join(input_file_name);

    // Run the backend binary
    run_backend(&output_binary, dist_dir)?;

    Ok(())
}

fn watch_files(
    roc_bin: &str,
    build_dir: &Path,
    dist_dir: &Path,
    input: &str,
    additional_paths: Vec<String>,
) -> Result<()> {
    // Create a channel to receive file system events
    let (tx, rx) = channel();

    // Create a watcher
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Determine the root directory of the input file to watch
    let input_path = Path::new(input);
    let input_dir = if input_path.is_absolute() {
        input_path.parent().unwrap().to_path_buf()
    } else {
        let current_dir = std::env::current_dir()?;
        current_dir.join(input_path).parent().unwrap().to_path_buf()
    };

    // Watch the input file's directory
    info!("Watching directory: {}", input_dir.display());
    watcher.watch(&input_dir, RecursiveMode::Recursive)?;

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

    // Create a thread to run the backend
    let input_clone = input.to_string();
    let roc_bin_clone = roc_bin.to_string();
    let build_dir_clone = build_dir.to_path_buf();
    let dist_dir_clone = dist_dir.to_path_buf();

    // Use a separate thread to run the backend
    thread::spawn(move || {
        if let Err(e) = {
            execute_build(
                &roc_bin_clone,
                &build_dir_clone,
                &dist_dir_clone,
                &input_clone,
            )
            .and_then(|_| execute_run(&build_dir_clone, &dist_dir_clone, &input_clone))
        } {
            error!("Command failed: {}", e);
            return;
        }
    });

    // Handle events with debouncing
    handle_events(rx, roc_bin, build_dir, dist_dir, input)?;

    Ok(())
}

fn handle_events(
    rx: Receiver<Result<Event, notify::Error>>,
    roc_bin: &str,
    build_dir: &Path,
    dist_dir: &Path,
    input: &str,
) -> Result<()> {
    let mut last_rebuild = Instant::now();

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                // Debounce events - only rebuild if a certain time has passed since the last rebuild
                let now = Instant::now();
                if now.duration_since(last_rebuild) > Duration::from_millis(DEBOUNCE_MS) {
                    debug!("File change detected: {:?}", event.paths);

                    // Rebuild the app
                    if let Err(e) = execute_build(roc_bin, build_dir, dist_dir, input) {
                        error!("Rebuild failed: {}", e);
                    } else {
                        debug!("Rebuild successful!");
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

fn build_wasm(roc_bin: &str, build_dir: &Path, input: &str) -> Result<()> {
    let status = build_wasm_cmd(roc_bin, build_dir.to_str().unwrap(), input)?
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

    Ok(())
}

fn copy_wasm_to_dist(build_dir: &Path, dist_dir: &Path, input: &str) -> Result<()> {
    let input_file_name = Path::new(Path::new(input).file_stem().unwrap()).with_extension("wasm");
    let source_wasm = build_dir.join(input_file_name);
    let dest_wasm = dist_dir.join("app.wasm");

    fs::copy(&source_wasm, &dest_wasm).context(format!(
        "Failed to copy WASM from {} to {}",
        source_wasm.to_str().unwrap(),
        dest_wasm.to_str().unwrap()
    ))?;

    Ok(())
}

fn build_backend(roc_bin: &str, build_dir: &Path, input: &str, output_binary: &Path) -> Result<()> {
    let status = build_backend_cmd(
        roc_bin,
        build_dir.to_str().unwrap(),
        input,
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

fn run_backend(output_binary: &Path, dist_dir: &Path) -> Result<()> {
    // Get absolute path to dist_dir for environment variable
    let dist_dir_abs = fs::canonicalize(dist_dir).context(format!(
        "Failed to get absolute path to {}",
        dist_dir.display()
    ))?;

    let mut run_cmd = Command::new(output_binary);
    run_cmd.env("DIST_DIR", dist_dir_abs.to_str().unwrap());

    info!("Running backend with DIST_DIR={}", dist_dir_abs.display());
    let status = run_cmd.status().context(format!(
        "Failed to execute backend binary {}",
        output_binary.display()
    ))?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Backend execution failed with status: {}",
            status
        ));
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

fn build_wasm_cmd(
    roc_bin_path: &str,
    build_dir: &str,
    source_file: &str,
) -> anyhow::Result<Command> {
    // We already validate file existence in execute_build
    create_directory_if_not_exists(Path::new(build_dir))?;

    let build_dir = if !build_dir.ends_with('/') {
        format!("{}/", build_dir)
    } else {
        build_dir.to_string()
    };

    let mut cmd = process::Command::new(roc_bin_path);
    cmd.args([
        "build",
        "--target",
        "wasm32",
        source_file,
        "--output",
        &build_dir,
    ]);

    Ok(cmd)
}

fn build_backend_cmd(
    _roc_bin_path: &str,
    build_dir: &str,
    source_file: &str,
    output_binary: &str,
) -> anyhow::Result<Command> {
    // NOTE: Not using the roc_bin_path for building the backend as it is only a workaround for
    // building with wasm where wasi-libc is required but not packaged when building with nix

    // We already validate file existence in execute_build
    create_directory_if_not_exists(Path::new(build_dir))?;

    let mut cmd = process::Command::new("roc");
    cmd.args([
        "build",
        "--emit-llvm-ir",
        source_file,
        "--output",
        output_binary,
    ]);

    Ok(cmd)
}
