use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Order {
    symbol: String,
    quantity: u32,
    order_type: OrderType,
    time_in_force: String, // "gtc", "ioc", etc.
}

// Response after placing an order
pub struct OrderResponse {
    pub id: String,
    pub status: OrderStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum OrderType {
    Buy,
    Sell,
}

// Example of order status
pub enum OrderStatus {
    Filled,
    Pending,
    Cancelled,
}
