use super::{
    asset::Asset,
    config::Config,
    order::{Order, OrderResponse},
};
use async_trait::async_trait;

#[async_trait]
pub trait TradingClient {
    fn new(config: &Config) -> Self
    where
        Self: Sized;
    async fn create_order(&self, order: &Order) -> Result<(), Box<dyn std::error::Error>>; // TODO: OrderResponse
    async fn get_asset(&self, symbol: &str) -> Result<Asset, Box<dyn std::error::Error>>;
    async fn subscribe_to_data(
        &self,
        symbol: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>>;
}
