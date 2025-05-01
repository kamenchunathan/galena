use std::{
    error::Error,
    fs,
    path::Path,
    process::{self, Command},
};

use anyhow::{self, Context, Result};
use clap::{Parser, Subcommand};
use include_dir::{include_dir, Dir};
use tracing::info;

static FRONTEND_DIR: Dir = include_dir!("$FRONTEND_DIST_DIR");

const GALENA_DIR: &str = ".galena";

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    let build_dir = Path::new(GALENA_DIR).join("build");
    let dist_dir = Path::new(GALENA_DIR).join("dist");
    let roc_bin = &cli.roc_bin.unwrap_or("roc".to_string());

    match cli.action {
        Action::Build { input } => {
            execute_build(roc_bin, &build_dir, &dist_dir, &input)?;
        }
        Action::Run { input } => {
            // First build the app (including WASM), since we need the dist directory
            execute_build(roc_bin, &build_dir, &dist_dir, &input)?;

            // Then build and run the backend
            execute_run(roc_bin, &build_dir, &dist_dir, &input)?;
        }
    }

    Ok(())
}

fn execute_build(roc_bin: &str, build_dir: &Path, dist_dir: &Path, input: &str) -> Result<()> {
    // Create dist directory and copy all frontend assets
    create_directory_if_not_exists(dist_dir)?;
    copy_frontend_to_dist(dist_dir)?;

    // Build WASM
    build_wasm(roc_bin, build_dir, input)?;

    // Copy WASM bundle to dist directory
    copy_wasm_to_dist(build_dir, dist_dir, input)?;

    info!(
        "Build completed. Files written to {}.",
        dist_dir.to_str().unwrap()
    );

    Ok(())
}

fn execute_run(roc_bin: &str, build_dir: &Path, dist_dir: &Path, input: &str) -> Result<()> {
    create_directory_if_not_exists(build_dir)?;

    let input_file_name = Path::new(input).file_stem().unwrap().to_str().unwrap();
    let output_binary = build_dir.join(input_file_name);

    // Build backend binary
    build_backend(roc_bin, build_dir, input, &output_binary)?;

    // Run the backend binary
    run_backend(&output_binary, dist_dir)?;

    Ok(())
}

fn build_wasm(roc_bin: &str, build_dir: &Path, input: &str) -> Result<()> {
    build_wasm_cmd(roc_bin, build_dir.to_str().unwrap(), input)?
        .status()
        .context("Unable to spawn roc build command")?;

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
    build_backend_cmd(
        roc_bin,
        build_dir.to_str().unwrap(),
        input,
        output_binary.to_str().unwrap(),
    )?
    .status()
    .context("Unable to spawn roc build command for backend")?;

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
    run_cmd.status().context(format!(
        "Failed to execute backend binary {}",
        output_binary.display()
    ))?;

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
    fs::metadata(source_file).context(format!(
        "Input file provided {} does not exist",
        source_file
    ))?;

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
    roc_bin_path: &str,
    build_dir: &str,
    source_file: &str,
    output_binary: &str,
) -> anyhow::Result<Command> {
    fs::metadata(source_file).context(format!(
        "Input file provided {} does not exist",
        source_file
    ))?;

    create_directory_if_not_exists(Path::new(build_dir))?;

    let mut cmd = process::Command::new(roc_bin_path);
    cmd.args(["build", source_file, "--output", output_binary]);

    Ok(cmd)
}
