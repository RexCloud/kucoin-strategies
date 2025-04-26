use serde::{Deserialize, Serialize};

use crate::kucoin::{constants::REDEEM, Request};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Redeem {
    currency: String,
    purchase_order_no: String,
    size: String,
}

impl Redeem {
    pub fn new(currency: String, purchase_order_no: String, size: f64) -> Self {
        Self {
            currency,
            purchase_order_no,
            size: size.to_string(),
        }
    }
}

impl From<Redeem> for Request {
    fn from(value: Redeem) -> Self {
        Request::post(REDEEM).json(&value)
    }
}
