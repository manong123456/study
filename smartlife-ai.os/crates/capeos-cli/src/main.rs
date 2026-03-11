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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_health() {
        let cli = Cli::try_parse_from(["capeos-cli", "health"]).unwrap();
        assert!(matches!(cli.command, Commands::Health));
    }

    #[test]
    fn test_cli_parse_version() {
        let cli = Cli::try_parse_from(["capeos-cli", "version"]).unwrap();
        assert!(matches!(cli.command, Commands::Version));
    }

    #[test]
    fn test_cli_parse_status() {
        let cli = Cli::try_parse_from(["capeos-cli", "status"]).unwrap();
        assert!(matches!(cli.command, Commands::Status));
    }

    #[test]
    fn test_cli_parse_custom_url() {
        let cli = Cli::try_parse_from(["capeos-cli", "--url", "http://localhost:8080", "health"]).unwrap();
        assert_eq!(cli.url, "http://localhost:8080");
        assert!(matches!(cli.command, Commands::Health));
    }
}
