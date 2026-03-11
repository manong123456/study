//! CapeOS CLI Diagnostic Tool
//!
//! Command-line utility for querying CapeOS backend health, version, and system status.
//! Connects to the gateway API to retrieve diagnostic information.

use clap::{Parser, Subcommand};

/// Top-level CLI arguments and subcommands.
#[derive(Parser)]
#[command(name = "capeos-cli", about = "CapeOS CLI diagnostic tool")]
struct Cli {
    /// Base URL of the CapeOS gateway (e.g., `http://localhost:80`).
    #[arg(short, long, default_value = "http://localhost:80")]
    url: String,

    /// Subcommand to execute.
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Check system health
    Health,
    /// Show system version
    Version,
    /// Show system utilization
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();

    match cli.command {
        Commands::Health => {
            let resp = client.get(format!("{}/v1/capeos/health/services", cli.url)).send().await?;
            println!("{}", resp.text().await?);
        }
        Commands::Version => {
            let resp = client.get(format!("{}/v1/sys/version/current", cli.url)).send().await?;
            println!("{}", resp.text().await?);
        }
        Commands::Status => {
            let resp = client.get(format!("{}/v1/sys/utilization", cli.url)).send().await?;
            println!("{}", resp.text().await?);
        }
    }
    Ok(())
}
