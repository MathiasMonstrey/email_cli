mod exchange;

use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::config::Config;

pub struct Email {
    pub id: String,
    pub subject: String,
    pub sender: String,
    pub date: DateTime<Utc>,
    pub body: String,
}

pub trait EmailClient {
    async fn fetch_current_quarter_emails(&self) -> Result<Vec<Email>>;
}

pub async fn create_client(config: &Config) -> Result<impl EmailClient> {
    exchange::ExchangeClient::new(&config.exchange).await
}
