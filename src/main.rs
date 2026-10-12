use anyhow::Result;
use clap::Parser;
use umami_reporter::{Cli, run};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(cli.verbosity.tracing_level_filter())
        .with_target(false)
        .init();

    run(cli).await
}
