use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub exchange: ExchangeConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExchangeConfig {
    pub email: String,
    pub password: String,
    pub server: String,
}

pub fn load_config(config_path: Option<PathBuf>) -> Result<Config> {
    // Create a new config builder
    let mut builder = config::Config::builder();
    
    // Set defaults
    builder = builder.set_default("exchange.server", "outlook.office365.com")?;
    
    // Try to load from specified path
    if let Some(path) = config_path {
        if path.exists() {
            builder = builder.add_source(config::File::from(path));
        }
    } else {
        // Try current directory
        if std::path::Path::new("config.toml").exists() {
            builder = builder.add_source(config::File::with_name("config"));
        }
        
        // Try home directory
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".config").join("mail-tui").join("config.toml");
            if config_path.exists() {
                builder = builder.add_source(config::File::from(config_path));
            }
        }
    }
    
    // Try environment variables
    builder = builder.add_source(config::Environment::with_prefix("MAIL_TUI"));
    
    // Build the config and convert to our Config struct
    let config = builder.build()?;
    config.try_deserialize().context("Failed to parse configuration")
}
