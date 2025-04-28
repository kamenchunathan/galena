fn main() {
    let platform_path = workspace_dir().join("frontend").join("dist");
    println!(
        "cargo:rustc-env=FRONTEND_DIST_DIR={}",
        platform_path.to_string_lossy()
    );
}

fn workspace_dir() -> std::path::PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = std::path::Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}
