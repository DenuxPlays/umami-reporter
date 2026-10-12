use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use lettre::message::Mailbox;

#[derive(Parser)]
#[command(
    name = "umami-reporter",
    about = "Versendet monatliche Umami-Statistiken per E-Mail"
)]
pub struct Cli {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    SendMonthlyReport(SendMonthlyReport),
}

#[derive(Args)]
pub struct SendMonthlyReport {
    #[arg(long = "recipient", short = 'r', required = true)]
    pub recipients: Vec<Mailbox>,
}
