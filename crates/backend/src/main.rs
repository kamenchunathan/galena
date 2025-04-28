use std::env;

use backend::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let v = env::var("DIST_DIR")?;
    run(v).await;

    Ok(())
}
