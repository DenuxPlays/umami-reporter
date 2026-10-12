mod cli;
mod config;
mod email;
mod report;

pub use cli::Cli;

use anyhow::Result;
use tracing::{debug, info};

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        cli::Command::SendMonthlyReport(args) => {
            info!("Lade Umami-Bericht");
            let config = config::Config::from_env()?;
            let (start, end) = report::previous_month();
            let report = report::fetch(&config, start, end).await?;
            debug!(
                month = %report.month,
                visitors = report.visitors,
                visits = report.visits,
                views = report.views,
                "Umami-Bericht geladen"
            );

            let rendered = email::render(&report)?;
            email::send(&config, &args.recipients, rendered)?;
            info!(month = %report.month, "Bericht wurde versendet");
        }
    }
    Ok(())
}
