use serde::{Deserialize, Serialize};

use crate::kucoin::{constants::LEND, Request};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lend {
    currency: String,
    interest_rate: String,
    size: String,
}

impl Lend {
    pub fn new(currency: String, interest_rate: f64, size: f64) -> Self {
        Self {
            currency,
            interest_rate: (interest_rate / 100.0).to_string(),
            size: size.to_string(),
        }
    }
}

impl From<Lend> for Request {
    fn from(value: Lend) -> Self {
        Request::post(LEND).json(&value)
    }
}
