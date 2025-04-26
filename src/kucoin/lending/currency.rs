use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    currency: String,
    purchase_enable: bool,
    redeem_enable: bool,
    increment: String,
    min_purchase_size: String,
    min_interest_rate: String,
    max_interest_rate: String,
    interest_increment: String,
    max_purchase_size: String,
    market_interest_rate: String,
    auto_purchase_enable: bool,
}

impl Currency {
    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn market_interest_rate(&self) -> f64 {
        self.market_interest_rate.parse::<f64>().unwrap() * 100.0
    }

    pub fn min_interest_rate(&self) -> f64 {
        self.min_interest_rate.parse::<f64>().unwrap() * 100.0
    }

    pub fn max_interest_rate(&self) -> f64 {
        self.max_interest_rate.parse::<f64>().unwrap() * 100.0
    }

    pub fn min_purchase_size(&self) -> f64 {
        self.min_purchase_size.parse().unwrap()
    }

    pub fn max_purchase_size(&self) -> f64 {
        self.max_purchase_size.parse().unwrap()
    }

    pub fn increment(&self) -> f64 {
        self.increment.parse().unwrap()
    }

    pub fn interest_increment(&self) -> f64 {
        self.interest_increment.parse().unwrap()
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<b>Coin:</b> {}\n\n\
            <b>Lending APY:</b> {}%\n\n\
            <b>Can subscribe:</b> {}\n\
            <b>Can redeem:</b> {}\n\n\
            <b>Lending APY Range:</b> {}% - {}%\n\
            <b>Lending Amount Range:</b> {} - {}\n\n\
            <b>Auto-Subscribe:</b> {}",
            self.currency,
            self.market_interest_rate(),
            self.purchase_enable,
            self.redeem_enable,
            self.min_interest_rate(),
            self.max_interest_rate(),
            self.min_purchase_size,
            self.max_purchase_size,
            self.auto_purchase_enable
        )
    }
}
