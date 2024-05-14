struct AccountStatus {
    balance: f32,
    positions: Vec<Position>,
}

struct Position {
    symbol: String,
    quantity: u32,
    average_price: f32,
}