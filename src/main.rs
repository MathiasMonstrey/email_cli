mod config;
mod email;
mod ui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Terminal UI for Office Exchange emails")]
struct Cli {
    /// Path to config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = config::load_config(cli.config)?;

    // Initialize email client
    let email_client = email::create_client(&config).await?;

    // Initialize and run the UI application
    let mut app = ui::app::App::new(email_client);
    app.run().await?;

    Ok(())
}
