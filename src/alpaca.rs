use crate::datastructures::asset::Asset;
use crate::datastructures::client::{FeedType, SubscriptionRequest, TradingClient};
use crate::datastructures::config::Config;
use crate::datastructures::order::Order;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::{header::HeaderMap, Client as HttpClient};
use serde_json::{json, to_string_pretty};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
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
        Ok(())
    }

    // async fn close_order();
    // async fn close_all_orders();
    async fn subscribe(
        &self,
        request: SubscriptionRequest,
        feed_type: FeedType,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> {
        let ws_url = get_ws_url(feed_type, self.enable_real_trading);
        let url = Url::parse(&ws_url)?;

        println!("key {:?}", self.api_key);
        println!("secret {:?}", self.secret_key);
        println!("base {:?}", self.base_url);
        println!("ws {:?}", ws_url);


        let (mut socket, response) = connect_async(url).await?;

        if response.status() != 101 {
            eprintln!("Failed to connect, status code: {}", response.status());
            return Err("Connection failed with non-101 status code".into());
        }
        println!("Connected to the server: {:?}", response);

        let auth_message = json!({
            "action": "auth",
            "key": self.api_key,
            "secret": self.secret_key
        });

        socket.send(Message::Text(auth_message.to_string())).await?;

        // Waiting for authentication response
        while let Some(message) = socket.next().await {
            match message? {
                Message::Text(text) => {
                    println!("Authentication response: {}", text);
                    if text.contains("unauthorized") || text.contains("error") {
                        return Err("Authentication failed".into());
                    } else if text.contains("authenticated") {
                        break; // Exit the loop if authentication is successful
                    }
                }
                _ => (), // Handle other message types if necessary
            }
        }

        let subscription_request = json!(request);

        socket
            .send(Message::Text(subscription_request.to_string()))
            .await?;

        Ok(socket)
    }

    /// Docs: https://docs.alpaca.markets/docs/streaming-market-data
    // async fn subscribe(
    //     &self,
    //     request: SubscriptionRequest,
    //     feed_type: FeedType,
    // ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> {
    //     let ws_url = get_ws_url(feed_type, self.enable_real_trading);
    //     let url = Url::parse(&ws_url)?;

    //     let (mut socket, response) = connect_async(url).await?;

    //     if response.status() != 101 {
    //         eprintln!("Failed to connect, status code: {}", response.status());
    //         return Err("Connection failed with non-101 status code".into());
    //     }

    //     println!("Connected to the server: {:?}", response);

    //     // Authenticate with your API key
    //     let auth_message = json!({
    //         "action": "authenticate",
    //         "data": {
    //             "key_id": self.api_key,
    //             "secret_key": self.secret_key
    //         }
    //     });

    //     socket.send(Message::Text(auth_message.to_string())).await?;

    //     // Waiting for authentication response
    //     if let Some(message) = socket.next().await {
    //         match message? {
    //             Message::Text(text) => {
    //                 println!("Authentication response: {}", text);
    //                 if text.contains("unauthorized") || text.contains("error") {
    //                     return Err("Authentication failed".into());
    //                 }
    //                 // Authentication successful, proceed with subscription
    //                 let subscription_request = json!(request);
    //                 let message_string = to_string_pretty(&subscription_request)?;
    //                 println!("Serialized JSON: {}", message_string);
    //                 socket
    //                     .send(Message::Binary(
    //                         subscription_request.to_string().into_bytes(),
    //                     ))
    //                     .await?;
    //             }
    //             _ => {
    //                 return Err("Unexpected authentication response".into());
    //             }
    //         }
    //     } else {
    //         return Err("No authentication response received".into());
    //     }

    //     // let subscription_request = json!(request);

    //     // let message_string = to_string_pretty(&subscription_request)?;
    //     // println!("Serialized JSON: {}", message_string);

    //     // socket
    //     // // .send(Message::Text(
    //     // //     subscription_request.to_string(),
    //     // // ))
    //     //     .send(Message::Binary(
    //     //         subscription_request.to_string().into_bytes(),
    //     //     ))
    //     //     .await?;

    //     Ok(socket)
    // }

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
