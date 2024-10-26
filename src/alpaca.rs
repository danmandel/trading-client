use crate::datastructures::{
    asset::Asset,
    client::{FeedType, SubscriptionParams, TradingClient},
    config::Config,
    order::Order,
};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::{header::HeaderMap, Client as HttpClient};
use serde_json::json;
use std::error::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use url::Url;

// Alpaca uses the same WebSocket API for both live and paper trading accounts when it comes to market data (IEX or SIP).
// The WebSocket endpoints for real-time market data do not differentiate between paper and live trading environments.
// The distinction between paper and live trading applies to order placement, not data streaming.
fn get_ws_url(feed_type: FeedType, enable_real_trading: bool) -> String {
    let src = "iex"; // "sip" requires subscription

    match feed_type {
        // Docs: https://docs.alpaca.markets/docs/real-time-stock-pricing-data
        FeedType::Stocks => {
            if enable_real_trading {
                format!("wss://stream.data.alpaca.markets/v2/{src}")
            } else {
                format!("wss://stream.data.alpaca.markets/v2/{src}")
            }
        }
        // Docs: https://docs.alpaca.markets/docs/real-time-crypto-pricing-data
        FeedType::Crypto => {
            if enable_real_trading {
                format!("wss://stream.data.alpaca.markets/v1beta3/crypto/us")
            } else {
                format!("wss://stream.data.alpaca.markets/v1beta3/crypto/us")
            }
        }
        // Docs: https://docs.alpaca.markets/docs/streaming-real-time-news
        FeedType::News => {
            if enable_real_trading {
                format!("wss://stream.data.alpaca.markets/v1beta1/news")
            } else {
                format!("wss://stream.data.sandbox.alpaca.markets/v1beta1/news")
            }
        }
        // Docs: https://docs.alpaca.markets/docs/real-time-option-data
        FeedType::Options => {
            if enable_real_trading {
                format!("wss://stream.data.alpaca.markets/v1beta1/{src}")
            } else {
                format!("wss://stream.data.sandbox.alpaca.markets/v1beta1/{src}")
            }
        }
        FeedType::Test => {
            if enable_real_trading {
                format!("wss://stream.data.alpaca.markets/v2/test")
            } else {
                format!("wss://stream.data.alpaca.markets/v2/test")
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

        // Insert order details into the postgres database. Consider using SQLite instead.
        // let (client, connection) =
        //     tokio_postgres::connect("host=localhost user=postgres password=secret dbname=orders", tokio_postgres::NoTls).await?;

        // // The connection object performs the actual communication with the database,
        // // so spawn it off to run on its own.
        // tokio::spawn(async move {
        //     if let Err(e) = connection.await {
        //         eprintln!("connection error: {}", e);
        //     }
        // });

        // client.execute(
        //     "INSERT INTO orders (symbol, quantity, order_type, time_in_force) VALUES ($1, $2, $3, $4)",
        //     &[&order.symbol, &order.quantity, &format!("{:?}", order.order_type), &order.time_in_force],
        // ).await?;

        Ok(())
    }

    // async fn close_order();
    // async fn close_all_orders();

    /// Docs: https://docs.alpaca.markets/docs/streaming-market-data
    async fn subscribe(
        &self,
        params: SubscriptionParams,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {
        let url = Url::parse(&get_ws_url(params.feed_type, self.enable_real_trading))?;

        let (mut socket, response) = connect_async(url).await?;

        if response.status() != 101 {
            return Err(
                format!("Connection failed with status code: {}", response.status()).into(),
            );
        }

        let auth_message = json!({
            "action": "auth",
            "key": self.api_key,
            "secret": self.secret_key
        });

        socket.send(Message::Text(auth_message.to_string())).await?;

        if let Some(message) = socket.next().await {
            match message? {
                Message::Text(text) => {
                    println!("Authentication Response: {}", text);
                    if text.contains("unauthorized") || text.contains("error") {
                        return Err("Authentication failed".into());
                    } else if !text.contains("success") {
                        return Err("Unexpected authentication response".into());
                    }
                }
                _ => {
                    return Err("Unexpected non-text message received during authentication".into())
                }
            }
        } else {
            return Err("No authentication response received".into());
        }

        socket
            .send(Message::Text(
                json!(params.subscription_request).to_string(),
            ))
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
