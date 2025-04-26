use std::fmt;

use crate::kucoin::KuCoin;

mod product;
pub use product::Product;

mod condition;
pub use condition::Condition;

mod actions;
pub use actions::{Action, ActionKind, Actions};

#[derive(Debug, Clone)]
pub struct Strategy {
    name: String,
    product: Option<Product>,
    condition: Option<Condition>,
    actions: Actions,
}

impl Strategy {
    pub fn new(name: String) -> Self {
        Self {
            name,
            product: Default::default(),
            condition: Default::default(),
            actions: Default::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn set_product(&mut self, product: Product) {
        self.product = Some(product)
    }

    pub fn set_condition(&mut self, condition: Condition) {
        self.condition = Some(condition)
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }

    pub fn actions_mut(&mut self) -> &mut Actions {
        &mut self.actions
    }

    pub fn can_execute(&self, kucoin: &KuCoin) -> bool {
        match self.product.as_ref().zip(self.condition.as_ref()) {
            Some((product, condition)) => {
                let maybe_value = match product {
                    Product::SpotTradingPair(symbol) => kucoin
                        .spot()
                        .tickers()
                        .get(symbol, false)
                        .and_then(|ticker| ticker.last()),
                    Product::LendingCurrency(currency) => kucoin
                        .lending()
                        .currencies()
                        .get(currency, false)
                        .map(|currency| currency.market_interest_rate()),
                    Product::BalanceCurrency { r#type, currency } => {
                        kucoin.accounts().available(r#type, currency)
                    }
                };

                match maybe_value {
                    Some(latest_value) => match condition {
                        Condition::GreaterThan(value) => latest_value > *value,
                        Condition::LessThan(value) => latest_value < *value,
                    },
                    None => false,
                }
            }
            None => false,
        }
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut actions = match self.actions.is_empty() {
            true => "🚫".to_string(),
            false => "\n".to_string(),
        };

        for (index, action) in self.actions.iter().enumerate() {
            actions.push_str(&format!("{}: {action}\n", index + 1));
        }

        write!(
            f,
            "<b>Name:</b> {}\n<b>Product:</b> {}\n<b>Condition:</b> {}\n<b>Actions:</b> {}",
            self.name,
            self.product
                .as_ref()
                .map_or_else(|| "🚫".to_string(), |product| product.to_string()),
            match (self.product.as_ref(), self.condition.as_ref()) {
                (Some(product), Some(condition)) => {
                    let reference = match product {
                        Product::SpotTradingPair(_) => "PRICE",
                        Product::LendingCurrency(_) => "APY",
                        Product::BalanceCurrency { .. } => "BALANCE",
                    };

                    format!("{reference} {condition}")
                }
                (_, Some(condition)) => condition.to_string(),
                _ => "🚫".to_string(),
            },
            actions
        )
    }
}
