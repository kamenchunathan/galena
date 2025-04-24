use std::{
    error::Error,
    fs,
    process::{self, Command},
};

use anyhow::{self, Context};
use clap::{Parser, Subcommand};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.acion {
        Action::Build { build_dir, input } => {
            build_wasm_cmd(
                cli.roc_bin.unwrap_or("roc".to_string()),
                build_dir.unwrap_or("out".to_string()),
                input,
            )?
            .status()
            .context("Unable to spawn roc command")?;
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
    acion: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    ///Builds input
    #[command()]
    Build {
        // Build / output directory
        #[arg(short = 'o', long)]
        build_dir: Option<String>,

        input: String,
    },
}

fn build_wasm_cmd(
    roc_bin_path: String,
    build_dir: String,
    source_file: String,
) -> anyhow::Result<Command> {
    fs::metadata(&build_dir).context(format!(
        "Build directory path provided {} does not exist",
        &build_dir
    ))?;
    fs::metadata(&source_file).context(format!(
        "Input file provided {} does not exist",
        &source_file
    ))?;

    let build_dir = if !source_file.ends_with('/') {
        format!("{build_dir}/")
    } else {
        build_dir
    };

    let mut cmd = process::Command::new(roc_bin_path);
    cmd.args([
        String::from("build"),
        String::from("--target"),
        String::from("wasm32"),
        source_file,
        String::from("--output"),
        build_dir,
    ]);

    Ok(cmd)
}
