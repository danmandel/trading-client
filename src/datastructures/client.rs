use super::{
    asset::Asset,
    config::Config,
    order::{Order, OrderResponse},
};
use async_trait::async_trait;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub enum FeedType {
    Stocks,
    Crypto,
    News,
    Options,
    Test,
}
#[derive(Serialize)]
pub struct SubscriptionRequest {
    /// Always "subscribe"
    pub action: &'static str,
    /// Array of ticker symbols ex. ["AAPL"] or ["BTC"]
    pub trades: Vec<&'static str>,
    // pub quotes: Vec<&'static str>,
    // pub bars: Vec<&'static str>,
    // pub updated_bars: Vec<&'static str>, // camelcase?
    // pub daily_bars: Vec<&'static str>,   // camelcase?
    // pub orderbooks: Vec<&'static str>,
}

pub struct SubscriptionRequestBuilder {
    trades: Vec<&'static str>,
    quotes: Vec<&'static str>,
    bars: Vec<&'static str>,
    updated_bars: Vec<&'static str>,
    daily_bars: Vec<&'static str>,
    orderbooks: Vec<&'static str>,
}

impl SubscriptionRequestBuilder {
    pub fn new() -> Self {
        SubscriptionRequestBuilder {
            trades: vec![],
            quotes: vec![],
            bars: vec![],
            updated_bars: vec![],
            daily_bars: vec![],
            orderbooks: vec![],
        }
    }

    pub fn trades(mut self, trades: &[&'static str]) -> Self {
        self.trades = trades.to_vec();
        self
    }

    pub fn quotes(mut self, quotes: &[&'static str]) -> Self {
        self.quotes = quotes.to_vec();
        self
    }

    pub fn bars(mut self, bars: &[&'static str]) -> Self {
        self.bars = bars.to_vec();
        self
    }

    pub fn updated_bars(mut self, updated_bars: &[&'static str]) -> Self {
        self.updated_bars = updated_bars.to_vec();
        self
    }

    pub fn daily_bars(mut self, daily_bars: &[&'static str]) -> Self {
        self.daily_bars = daily_bars.to_vec();
        self
    }

    pub fn orderbooks(mut self, orderbooks: &[&'static str]) -> Self {
        self.orderbooks = orderbooks.to_vec();
        self
    }

    pub fn build(self) -> SubscriptionRequest {
        SubscriptionRequest {
            action: "subscribe",
            trades: self.trades,
            // quotes: self.quotes,
            // bars: self.bars,
            // updated_bars: self.updated_bars,
            // daily_bars: self.daily_bars,
            // orderbooks: self.orderbooks,
        }
    }
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
        request: SubscriptionRequest,
        feed_type: FeedType,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>>;
}
