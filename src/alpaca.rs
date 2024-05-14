use crate::datastructures::asset::Asset;
use crate::datastructures::client::TradingClient;
use crate::datastructures::config::Config;
use crate::datastructures::order::Order;
use async_trait::async_trait;
use reqwest::{header::HeaderMap, Client as HttpClient};

pub struct AlpacaClient {
    http_client: HttpClient,
    base_url: String,
    api_key: String,
    secret_key: String,
}

#[async_trait]
impl TradingClient for AlpacaClient {
    fn new(config: &Config) -> Self {
        AlpacaClient {
            http_client: HttpClient::new(),
            base_url: config.alpaca_base_url.clone(),
            api_key: config.alpaca_api_key.clone(),
            secret_key: config.alpaca_secret_key.clone(),
        }
    }

    async fn create_order(&self, order: &Order) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/v2/orders", self.base_url);
        let mut headers = HeaderMap::new();
        headers.insert("APCA-API-KEY-ID", self.api_key.parse()?);
        headers.insert("APCA-API-SECRET-KEY", self.secret_key.parse()?);
        headers.insert("accept", "application/json".parse()?);

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(&order)
            .send()
            .await?
            .text()
            .await?;

        println!("Create Order Response: {}", response);
        Ok(())
    }

    async fn get_asset(&self, symbol: &str) -> Result<Asset, Box<dyn std::error::Error>> {
        let url = format!("{}/v2/assets/{}", self.base_url, symbol);
        let mut headers = HeaderMap::new();
        headers.insert("APCA-API-KEY-ID", self.api_key.parse()?);
        headers.insert("APCA-API-SECRET-KEY", self.secret_key.parse()?);
        headers.insert("accept", "application/json".parse()?);

        let response = self
            .http_client
            .get(&url)
            .headers(headers)
            .send()
            .await?
            .text()
            .await?;

        println!("Get Asset Response: {}", response);

        let asset: Asset = serde_json::from_str(&response)?;
        Ok(asset)
    }
}
