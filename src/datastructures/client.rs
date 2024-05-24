use super::{
    asset::Asset,
    config::Config,
    order::{Order, OrderResponse},
};
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio_tungstenite::{ MaybeTlsStream, WebSocketStream};

pub enum FeedType {
    Stocks,
    Crypto,
    News,
    Options,
}

#[async_trait]
pub trait TradingClient {
    fn new(config: &Config) -> Self
    where
        Self: Sized;
    async fn create_order(&self, order: &Order) -> Result<(), Box<dyn std::error::Error>>; // TODO: OrderResponse
    async fn get_asset(&self, symbol: &str) -> Result<Asset, Box<dyn std::error::Error>>;
    async fn subscribe(
        &self,
        symbol: &str,
        feed_type: FeedType,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>>;
}
