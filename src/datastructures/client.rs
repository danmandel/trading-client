use super::{
    asset::Asset,
    config::Config,
    order::{Order, OrderResponse},
};
use async_trait::async_trait;
use futures_util::sink::Feed;

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

    //pub async fn subscribe<S>(&self) -> Result<(S::Stream, S::Subscription), Error>
    async fn subscribe(
        &self,
        symbol: &str,
        feed_type: FeedType,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
