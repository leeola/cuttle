use anyhow::Result;
use cuttle_bin::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
