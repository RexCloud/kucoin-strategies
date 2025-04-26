use std::fmt;

use crate::kucoin::account::AccountType;

#[derive(Debug, Clone)]
pub enum Product {
    SpotTradingPair(String),
    LendingCurrency(String),
    BalanceCurrency {
        r#type: AccountType,
        currency: String,
    },
}

impl fmt::Display for Product {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SpotTradingPair(pair) => write!(f, "{pair} (SPOT TRADING)"),
            Self::LendingCurrency(currency) => write!(f, "{currency} (LENDING)"),
            Self::BalanceCurrency { r#type, currency } => {
                write!(
                    f,
                    "{currency} ({} ACCOUNT)",
                    r#type.to_string().to_uppercase()
                )
            }
        }
    }
}
