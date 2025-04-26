use std::fmt;

use crate::kucoin::{
    account::AccountType,
    trading::spot::order::{Side, Type},
    KuCoin,
};

#[derive(Debug, Clone)]
pub struct Action {
    kind: ActionKind,
    symbol: String,
    percentage: u8,
    skip: bool,
}

#[derive(Debug, Clone)]
pub enum ActionKind {
    SpotOrder {
        r#type: Type,
        side: Side,
        price: Option<f64>,
    },
    Lend {
        interest_rate: f64,
    },
    Redeem,
    Transfer {
        from: AccountType,
        to: AccountType,
        from_account_tag: Option<String>,
        to_account_tag: Option<String>,
    },
}

impl Action {
    fn new(kind: ActionKind) -> Self {
        Action {
            kind,
            symbol: Default::default(),
            percentage: Default::default(),
            skip: true,
        }
    }

    fn spot_order(side: Side) -> Self {
        Action::new(ActionKind::SpotOrder {
            side,
            r#type: Default::default(),
            price: Default::default(),
        })
    }

    pub fn buy() -> Self {
        Action::spot_order(Side::Buy)
    }

    pub fn sell() -> Self {
        Action::spot_order(Side::Sell)
    }

    pub fn lend() -> Self {
        Action::new(ActionKind::Lend {
            interest_rate: Default::default(),
        })
    }

    pub fn redeem() -> Self {
        Action::new(ActionKind::Redeem)
    }

    pub fn transfer() -> Self {
        Action::new(ActionKind::Transfer {
            from: Default::default(),
            to: Default::default(),
            from_account_tag: Default::default(),
            to_account_tag: Default::default(),
        })
    }

    pub fn kind(&self) -> &ActionKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut ActionKind {
        &mut self.kind
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn symbol_mut(&mut self) -> &mut String {
        &mut self.symbol
    }

    pub fn percentage(&self) -> u8 {
        self.percentage
    }

    pub fn percentage_mut(&mut self) -> &mut u8 {
        &mut self.percentage
    }

    pub fn skip(&self) -> bool {
        self.skip
    }

    pub fn skip_mut(&mut self) -> &mut bool {
        &mut self.skip
    }

    pub fn amount(&self, kucoin: &KuCoin) -> Option<f64> {
        let percentage = self.percentage() as f64 / 100.0;

        match &self.kind {
            ActionKind::SpotOrder { r#type, side, .. } => kucoin
                .spot()
                .symbols()
                .get(self.symbol())
                .map(|symbol| {
                    if let (Type::Market, Side::Buy) = (r#type, side) {
                        (
                            symbol.quote_currency(),
                            symbol.quote_increment(),
                            symbol.quote_min_size(),
                            symbol.quote_max_size(),
                        )
                    } else {
                        (
                            symbol.base_currency(),
                            symbol.base_increment(),
                            symbol.base_min_size(),
                            symbol.base_max_size(),
                        )
                    }
                })
                .and_then(|(currency, increment, min_size, max_size)| {
                    kucoin
                        .accounts()
                        .available(&AccountType::Trade, currency)
                        .map(|amount| amount * percentage)
                        .map(|amount| with_increment(amount, increment))
                        .filter(|amount| (min_size..=max_size).contains(amount))
                }),
            ActionKind::Lend { .. } => kucoin
                .lending()
                .currencies()
                .get(self.symbol(), false)
                .and_then(|currency| {
                    kucoin
                        .accounts()
                        .available(&AccountType::Main, self.symbol())
                        .map(|amount| amount * percentage)
                        .map(|amount| with_increment(amount, currency.increment()))
                        .filter(|amount| {
                            (currency.min_purchase_size()..=currency.max_purchase_size())
                                .contains(amount)
                        })
                }),
            ActionKind::Redeem => kucoin
                .lending()
                .currencies()
                .get(self.symbol(), false)
                .map(|currency| currency.increment())
                .and_then(|increment| {
                    kucoin
                        .lending()
                        .orders()
                        .get(self.symbol())
                        .map(|order| order.purchase_size() * percentage)
                        .map(|amount| with_increment(amount, increment))
                        .filter(|amount| *amount >= increment)
                }),
            ActionKind::Transfer { from, .. } => kucoin
                .spot()
                .currencies()
                .get(self.symbol())
                .map(|currency| 10_f64.powi(currency.precision().into()))
                .and_then(|precision| {
                    kucoin
                        .accounts()
                        .available(from, self.symbol())
                        .map(|amount| amount * percentage)
                        .map(|amount| (amount * precision).trunc() / precision)
                        .filter(|amount| *amount >= precision.recip())
                }),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = self.symbol();
        let percentage = self.percentage();

        match &self.kind {
            ActionKind::SpotOrder {
                r#type,
                side,
                price,
            } => {
                write!(
                    f,
                    "{} {} {symbol} {percentage}% {}",
                    r#type,
                    side,
                    price.map_or(Default::default(), |price| format!("@ {price}"))
                )
            }
            ActionKind::Lend { interest_rate } => write!(
                f,
                "LEND {symbol} {percentage}% MIN LENDING APY {}%",
                interest_rate
            ),
            ActionKind::Redeem => write!(f, "REDEEM {symbol} {percentage}%"),
            ActionKind::Transfer {
                from,
                to,
                from_account_tag,
                to_account_tag,
            } => write!(
                f,
                "TRANSFER {symbol} {percentage}% ({} {} -> {} {})",
                from.to_string().to_uppercase(),
                from_account_tag.as_ref().map_or("", |v| v),
                to.to_string().to_uppercase(),
                to_account_tag.as_ref().map_or("", |v| v),
            ),
        }
    }
}

fn with_increment(value: f64, increment: f64) -> f64 {
    let decimals = increment.log10().abs() as i32;
    let precision = 10_f64.powi(decimals);
    (value * precision).trunc() / precision
}
