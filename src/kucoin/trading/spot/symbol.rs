use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    symbol: String,
    name: String,
    base_currency: String,
    quote_currency: String,
    fee_currency: String,
    market: String,
    base_min_size: String,
    quote_min_size: String,
    base_max_size: String,
    quote_max_size: String,
    base_increment: String,
    quote_increment: String,
    price_increment: String,
    price_limit_rate: String,
    min_funds: Option<String>,
    is_margin_enabled: bool,
    enable_trading: bool,
    fee_category: u8,
    maker_fee_coefficient: String,
    taker_fee_coefficient: String,
    st: bool,
    callauction_is_enabled: bool,
    callauction_price_floor: Option<String>,
    callauction_price_ceiling: Option<String>,
    callauction_first_stage_start_time: Option<u64>,
    callauction_second_stage_start_time: Option<u64>,
    callauction_third_stage_start_time: Option<u64>,
    trading_start_time: Option<u64>,
}

impl Symbol {
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn base_currency(&self) -> &str {
        &self.base_currency
    }

    pub fn quote_currency(&self) -> &str {
        &self.quote_currency
    }

    pub fn base_min_size(&self) -> f64 {
        self.base_min_size.parse().unwrap()
    }

    pub fn quote_min_size(&self) -> f64 {
        self.quote_min_size.parse().unwrap()
    }

    pub fn base_max_size(&self) -> f64 {
        self.base_max_size.parse().unwrap()
    }

    pub fn quote_max_size(&self) -> f64 {
        self.quote_max_size.parse().unwrap()
    }

    pub fn base_increment(&self) -> f64 {
        self.base_increment.parse().unwrap()
    }

    pub fn quote_increment(&self) -> f64 {
        self.base_increment.parse().unwrap()
    }

    pub fn price_increment(&self) -> f64 {
        self.price_increment.parse().unwrap()
    }
}
