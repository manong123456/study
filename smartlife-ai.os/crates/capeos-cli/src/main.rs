use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "capeos-cli", about = "CapeOS CLI diagnostic tool")]
struct Cli {
    #[arg(short, long, default_value = "http://localhost:80")]
    url: String,

    #[command(subcommand)]
    command: Commands,
}

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
