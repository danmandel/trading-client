use crate::datastructures::asset::Asset;
use crate::datastructures::client::{FeedType, TradingClient};
use crate::datastructures::config::Config;
use crate::datastructures::order::Order;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::{header::HeaderMap, Client as HttpClient};
use serde_json::{json, to_string_pretty};
use tokio::net::TcpStream;
use tokio_tungstenite::{ MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

pub struct SubScriptionRequest {
    pub action: &'static str,
    pub trades: Vec<&'static str>,
    pub quotes: Vec<&'static str>,
    pub bars: Vec<&'static str>,
    pub updated_bars: Vec<&'static str>, // camelcase?
    pub daily_bars: Vec<&'static str>, // camelcase?
    pub orderbooks: Vec<&'static str>
}

fn get_ws_url(feed_type: FeedType, enable_real_trading: bool) -> &'static str {
    match feed_type {
        // Docs: https://docs.alpaca.markets/docs/real-time-stock-pricing-data
        FeedType::Stocks => {
            if enable_real_trading {
                "wss://api.alpaca.markets/stream" // "wss://stream.data.alpaca.markets/v2/iex"
            } else {
                //"wss://stream.data.alpaca.markets/v2/test" 
                "wss://paper-api.alpaca.markets/stream" // "wss://paper-stream.data.alpaca.markets/v2/iex"
            }
        }
        // Docs: https://docs.alpaca.markets/docs/real-time-crypto-pricing-data
        FeedType::Crypto => {
            if enable_real_trading {
                "wss://stream.data.alpaca.markets/v1beta3/crypto/us"
            } else {
                "wss://stream.data.sandbox.alpaca.markets/v1beta3/crypto/us"
            }
        }
        // Docs: https://docs.alpaca.markets/docs/streaming-real-time-news
        FeedType::News => {
            if enable_real_trading {
                "wss://stream.data.alpaca.markets/v1beta1/news"
            } else {
                "wss://stream.data.sandbox.alpaca.markets/v1beta1/news"
            }
        }
        // Docs: https://docs.alpaca.markets/docs/real-time-option-data
        FeedType::Options => {
            if enable_real_trading {
                "wss://stream.data.alpaca.markets/v1beta1/{feed}"
            } else {
                "wss://stream.data.sandbox.alpaca.markets/v1beta1/{feed}" // TODO: Substitute indicative or opra to {feed} depending on your subscription.
            }
        }
    }
}

#[derive(Clone)]
pub struct AlpacaClient {
    http_client: HttpClient,
    base_url: &'static str,
    api_key: String,
    secret_key: String,
    enable_real_trading: bool,
    // cfg: Config, TODO: possibly cleaner to put the entire config object on the client instead of manually adding each property.
}

#[async_trait]
impl TradingClient for AlpacaClient {
    fn new(config: &Config) -> Self {
        let base_url = if config.enable_real_trading {
            "https://api.alpaca.markets"
        } else {
            "https://paper-api.alpaca.markets"
        };

        AlpacaClient {
            http_client: HttpClient::new(),
            base_url,
            api_key: config.alpaca_api_key.clone(),
            secret_key: config.alpaca_secret_key.clone(),
            enable_real_trading: config.enable_real_trading,
        }
    }
 
    // TODO: what if order was its own struct that had adjust_for_confidence and adjust_for_kelly_criteron
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

    // async fn close_order();
    // async fn close_all_orders();

    /// Docs: https://docs.alpaca.markets/docs/streaming-market-data
    async fn subscribe(
        &self,
        symbol: &str,
        feed_type: FeedType,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> {
        let ws_url = get_ws_url(feed_type, self.enable_real_trading);
        let url = Url::parse(ws_url)?;

        let (mut socket, response) = connect_async(url).await?;

        if response.status() != 101 {
            eprintln!("Failed to connect, status code: {}", response.status());
            return Err("Connection failed with non-101 status code".into());
        }

        println!("Connected to the server: {:?}", response);

        // Authenticate with your API key
        let auth_message = json!({
            "action": "authenticate",
            "data": {
                "key_id": self.api_key,
                "secret_key": self.secret_key  // Ensure these are correctly formatted
            }
        });

        socket.send(Message::Text(auth_message.to_string())).await?;

        // Waiting for authentication response
        if let Some(message) = socket.next().await {
            match message? {
                Message::Text(text) => {
                    println!("Authentication response: {}", text);
                    if text.contains("unauthorized") || text.contains("error") {
                        return Err("Authentication failed".into());
                    }
                }
                _ => (),
            }
        }


        // Subscribe to minutely updates for the specified symbol.
        let subscribe_message = json!({
            "action": "subscribe",
            "bars": ["AAPL"],
            "trades": ["AAPL"],
            "quotes": ["AAPL"],
            "daily_bars": ["AAPL"]            
        });

        let message_string = to_string_pretty(&subscribe_message)?;
        println!("Serialized JSON: {}", message_string);

        socket
            .send(Message::Text(subscribe_message.to_string()))
            .await?;

        Ok(socket)
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
