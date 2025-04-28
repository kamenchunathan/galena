use std::{
    error::Error,
    fs,
    path::Path,
    process::{self, Command},
};

use anyhow::{self, Context, Result};
use clap::{Parser, Subcommand};
use tracing::info;

const INDEX_HTML: &str = include_str!("../../../frontend/dist/index.html");
const INDEX_JS: &str = include_str!("../../../frontend/dist/assets/index-CZVMipa3.js");

const GALENA_DIR: &str = ".galena";

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    let build_dir = Path::new(GALENA_DIR).join("build");
    let dist_dir = Path::new(GALENA_DIR).join("dist");
    let roc_bin = &cli.roc_bin.unwrap_or("roc".to_string());

    match cli.action {
        Action::Build { input } => {
            dbg!(build_wasm_cmd(
                &roc_bin,
                build_dir.to_str().unwrap(),
                &input,
            )?)
            .status()
            .context("Unable to spawn roc build command")?;

            create_directory_if_not_exists(&dist_dir)?;

            fs::write(Path::new(&dist_dir).join("index.html"), INDEX_HTML)
                .context("Failed to write index.html to dist directory")?;
            fs::write(Path::new(&dist_dir).join("index.js"), INDEX_JS)
                .context("Failed to write index.js to dist directory")?;

            // Copy WASM bundle to dist directory
            let input_file_name =
                Path::new(Path::new(&input).file_stem().unwrap()).with_extension("wasm");
            let source_wasm = build_dir.join(input_file_name);
            let dest_wasm = dist_dir.join("app.wasm");
            fs::copy(&source_wasm, &dest_wasm).context(format!(
                "Failed to copy WASM from {} to {}",
                source_wasm.to_str().unwrap(),
                dest_wasm.to_str().unwrap()
            ))?;

            info!(
                "Build completed. Files written to {}.",
                dist_dir.to_str().unwrap()
            );
        }
        Action::Run { input } => {
            create_directory_if_not_exists(&build_dir)?;

            let input_file_name = Path::new(&input).file_stem().unwrap().to_str().unwrap();
            let output_binary = build_dir.join(input_file_name);

            build_backend_cmd(
                &roc_bin,
                build_dir.to_str().unwrap(),
                &input,
                output_binary.to_str().unwrap(),
            )?
            .status()
            .context("Unable to spawn roc build command for backend")?;

            // Run the backend binary with DIST_DIR environment variable
            let dist_dir = fs::canonicalize("dist")
                .context("Failed to get absolute path to dist directory")?;

            let mut run_cmd = Command::new(&output_binary);
            run_cmd.env("DIST_DIR", dist_dir.to_str().unwrap());

            println!(
                "Running backend with DIST_DIR={}",
                dist_dir.to_str().unwrap()
            );
            run_cmd.status().context(format!(
                "Failed to execute backend binary {}",
                output_binary.to_str().unwrap()
            ))?;
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
    fs::metadata(&source_file).context(format!(
        "Input file provided {} does not exist",
        &source_file
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
    fs::metadata(&source_file).context(format!(
        "Input file provided {} does not exist",
        &source_file
    ))?;

    create_directory_if_not_exists(Path::new(build_dir))?;

    let mut cmd = process::Command::new(roc_bin_path);
    cmd.args(["build", source_file, "--output", output_binary]);

    Ok(cmd)
}
