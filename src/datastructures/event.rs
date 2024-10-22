#[derive(Debug, Clone)]
pub enum EventType {
    Trade {
        symbol: String,
        price: f64,
        volume: u64,
        timestamp: String,
    },
    Quote {
        symbol: String,
        bid_price: f64,
        ask_price: f64,
        bid_size: u64,
        ask_size: u64,
        timestamp: String,
    },
    Bar {
        symbol: String,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: u64,
        timestamp: String,
    },
    UpdatedBar {
        symbol: String,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: u64,
        timestamp: String,
    },
    DailyBar {
        symbol: String,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: u64,
        timestamp: String,
    },
    OrderBook {
        symbol: String,
        bids: Vec<(f64, u64)>, // (price, size)
        asks: Vec<(f64, u64)>, // (price, size)
        timestamp: String,
    },
}

use serde::de::Error as SerdeError;
use serde::Deserialize;
use serde_json::Error;

impl EventType {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        #[derive(Deserialize)]
        struct RawEvent {
            T: String,
            S: String,
            bp: Option<f64>,
            bs: Option<f64>,
            ap: Option<f64>,
            as_: Option<f64>,
            o: Option<f64>,
            h: Option<f64>,
            l: Option<f64>,
            c: Option<f64>,
            v: Option<u64>,
            t: String,
            bids: Option<Vec<(f64, u64)>>,
            asks: Option<Vec<(f64, u64)>>,
        }

        let raw_event: Vec<RawEvent> = serde_json::from_str(s)?;

        if raw_event.is_empty() {
            return Err(SerdeError::custom("Empty event list"));
        }

        let event = &raw_event[0];

        match event.T.as_str() {
            "q" => Ok(EventType::Quote {
                symbol: event.S.clone(),
                bid_price: event.bp.unwrap_or_default(),
                ask_price: event.ap.unwrap_or_default(),
                bid_size: event.bs.unwrap_or_default() as u64,
                ask_size: event.as_.unwrap_or_default() as u64,
                timestamp: event.t.clone(),
            }),
            "b" => Ok(EventType::Bar {
                symbol: event.S.clone(),
                open: event.o.unwrap_or_default(),
                high: event.h.unwrap_or_default(),
                low: event.l.unwrap_or_default(),
                close: event.c.unwrap_or_default(),
                volume: event.v.unwrap_or_default(),
                timestamp: event.t.clone(),
            }),
            _ => Err(SerdeError::custom("Unknown event type")),
        }
    }
}
