use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    currency: String,
    purchase_order_no: String,
    purchase_size: String,
    match_size: String,
    interest_rate: String,
    income_size: String,
    apply_time: u64,
    status: String,
}

impl Order {
    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn purchase_order_no(&self) -> &str {
        &self.purchase_order_no
    }

    pub fn purchase_size(&self) -> f64 {
        self.purchase_size.parse().unwrap()
    }

    fn interest_rate(&self) -> f32 {
        self.interest_rate.parse::<f32>().unwrap() * 100.0
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<b>Coin:</b> {}\n\
            <b>Min lending APY:</b> {}%\n\
            <b>Lending amount:</b> {}\n\
            <b>Lent out:</b> {}\n\
            <b>Total earnings:</b> {}\n\n",
            self.currency,
            self.interest_rate(),
            self.purchase_size,
            self.match_size,
            self.income_size
        )
    }
}
