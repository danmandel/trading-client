use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Asset {
    pub symbol: String,
    pub exchange: String,
}
