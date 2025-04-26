use serde::{Deserialize, Serialize};
use std::fmt;

use crate::kucoin::{constants::SPOT_ORDER, Request};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Add {
    r#type: Type,
    symbol: String,
    side: Side,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_oid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stp: Option<SelfTradePrevention>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cancel_after: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    post_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    iceberg: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visible_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    funds: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    #[default]
    Limit,
    Market,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Limit => write!(f, "LIMIT"),
            Type::Market => write!(f, "MARKET"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    #[default]
    Buy,
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum SelfTradePrevention {
    Dc,
    Co,
    Cn,
    Cb,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum TimeInForce {
    Gtc,
    Gtt,
    Ioc,
    Fok,
}

impl Add {
    pub fn limit(symbol: String, side: Side, price: f64, size: f64) -> Self {
        Self {
            r#type: Type::Limit,
            symbol,
            side,
            price: Some(price.to_string()),
            size: Some(size.to_string()),
            ..Default::default()
        }
    }

    pub fn market(symbol: String, side: Side, amount: f64) -> Self {
        let amount = amount.to_string();

        let (size, funds) = match side {
            Side::Buy => (None, Some(amount)),
            Side::Sell => (Some(amount), None),
        };

        Self {
            r#type: Type::Market,
            symbol,
            side,
            size,
            funds,
            ..Default::default()
        }
    }
}

impl From<Add> for Request {
    fn from(value: Add) -> Self {
        Request::post(SPOT_ORDER).json(&value)
    }
}
