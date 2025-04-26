use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Ticker {
    symbol: String,
    symbol_name: String,
    buy: Option<String>,
    best_bid_size: Option<String>,
    sell: Option<String>,
    best_ask_size: Option<String>,
    change_rate: Option<String>,
    change_price: Option<String>,
    high: String,
    low: String,
    vol: Option<String>,
    vol_value: String,
    last: Option<String>,
    average_price: Option<String>,
    taker_fee_rate: String,
    maker_fee_rate: String,
    taker_coefficient: String,
    maker_coefficient: String,
}

impl Ticker {
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn last(&self) -> Option<f64> {
        self.last.as_ref().and_then(|last| last.parse().ok())
    }

    fn change_rate(&self) -> Option<f64> {
        self.change_rate
            .as_ref()
            .and_then(|change_rate| change_rate.parse().ok())
            .map(|change_rate: f64| change_rate * 100.0)
    }

    fn taker_fee_rate(&self) -> f32 {
        self.taker_fee_rate.parse::<f32>().unwrap() * 100.0
    }

    fn maker_fee_rate(&self) -> f32 {
        self.maker_fee_rate.parse::<f32>().unwrap() * 100.0
    }

    fn taker_coefficient(&self) -> f32 {
        self.taker_coefficient.parse().unwrap()
    }

    fn maker_coefficient(&self) -> f32 {
        self.maker_coefficient.parse().unwrap()
    }
}

impl fmt::Display for Ticker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<b>Pair:</b> {}\n\n\
            <b>Price:</b> {} ( {}% / {} )\n\n\
            <b>24h High:</b> {}\n\
            <b>24h Low:</b> {}\n\
            <b>24h Avg Price:</b> {}\n\
            <b>24h Volume:</b> {}\n\n\
            <b>Order Book:</b>\n\
            ...\n\
            {} ( {} )\n\
            ===\n\
            {} ( {} )\n\
            ...\n\n\
            <b>Taker Fee:</b> {}%\n\
            <b>Maker Fee:</b> {}%",
            self.symbol,
            self.last.as_ref().unwrap_or(&"0".to_string()),
            self.change_rate().unwrap_or_default(),
            self.change_price.as_ref().unwrap_or(&"0".to_string()),
            self.high,
            self.low,
            self.average_price.as_ref().unwrap_or(&"0".to_string()),
            self.vol_value,
            self.sell.as_ref().unwrap_or(&"0".to_string()),
            self.best_ask_size.as_ref().unwrap_or(&"0".to_string()),
            self.buy.as_ref().unwrap_or(&"0".to_string()),
            self.best_bid_size.as_ref().unwrap_or(&"0".to_string()),
            self.taker_fee_rate() * self.taker_coefficient(),
            self.maker_fee_rate() * self.maker_coefficient(),
        )
    }
}
