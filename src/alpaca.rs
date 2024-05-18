use crate::datastructures::asset::Asset;
use crate::datastructures::client::TradingClient;
use crate::datastructures::config::Config;
use crate::datastructures::order::Order;
use async_trait::async_trait;
use reqwest::{header::HeaderMap, Client as HttpClient};

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url; // Importing the required traits

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

    async fn subscribe_to_data(&self, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
        let api_url = format!("wss://data.alpaca.markets/stream");
        let url = Url::parse(&api_url).expect("Invalid URL");

        let (mut socket, response) = connect_async(url).await.expect("Failed to connect");
        println!("Connected to the server: {:?}", response);

        // Authenticate with your API key
        let auth_message = json!({
            "action": "authenticate",
            "data": {
                "key_id": self.api_key,
                "secret_key": self.secret_key  // Adjust according to actual required authentication schema
            }
        });
        socket.send(Message::Text(auth_message.to_string())).await?;

        // Subscribe to minutely updates for the specified symbol
        let subscribe_message = json!({
            "action": "listen",
            "data": {
                "streams": [format!("AM.{}", symbol)]
            }
        });
        socket
            .send(Message::Text(subscribe_message.to_string()))
            .await?;

        // Listening to the messages
        while let Some(message) = socket.next().await {
            let msg = message?;
            match msg {
                Message::Text(text) => println!("Received: {}", text),
                Message::Binary(bin) => println!("Received binary data: {:?}", bin),
                _ => (),
            }
        }

        Ok(())
    }

    // async fn subscribe_to_data(
    //     api_key: &str,
    //     symbol: &str,
    // ) -> tokio_tungstenite::tungstenite::Result<()> {
    //     let api_url = format!("wss://data.alpaca.markets/stream");
    //     let url = Url::parse(&api_url).expect("Invalid URL");

    //     let (mut socket, response) = connect_async(url).await.expect("Failed to connect");
    //     println!("Connected to the server: {:?}", response);

    //     // Authenticate with your API key
    //     let auth_message = json!({
    //         "action": "authenticate",
    //         "data": {
    //             "key_id": api_key,
    //             "secret_key": api_key  // Adjust according to actual required authentication schema
    //         }
    //     });
    //     socket.send(Message::Text(auth_message.to_string())).await?;

    //     // Subscribe to minutely updates for the specified symbol
    //     let subscribe_message = json!({
    //         "action": "listen",
    //         "data": {
    //             "streams": [format!("AM.{}", symbol)]
    //         }
    //     });
    //     socket
    //         .send(Message::Text(subscribe_message.to_string()))
    //         .await?;

    //     // Listening to the messages
    //     while let Some(message) = socket.next().await {
    //         let msg = message?;
    //         match msg {
    //             Message::Text(text) => println!("Received: {}", text),
    //             Message::Binary(bin) => println!("Received binary data: {:?}", bin),
    //             _ => (),
    //         }
    //     }

    //     Ok(())
    // }
}
