use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    str::FromStr,
    sync::{Arc, Mutex, MutexGuard},
};
use tracing::error;

use crate::kucoin::{constants::ACCOUNTS, task::Poller, Request};

mod transfer;
pub use transfer::Transfer;

#[derive(Debug, Default, Clone)]
pub struct Accounts(Arc<Mutex<HashMap<AccountType, Vec<Account>>>>);

impl Accounts {
    pub fn available(&self, r#type: &AccountType, currency: &str) -> Option<f64> {
        self.lock().get(r#type).and_then(|accounts| {
            accounts
                .iter()
                .find(|account| account.currency() == currency)
                .map(|account| account.available())
        })
    }

    fn set(&self, accounts: Vec<Account>) {
        let mut lock = self.lock();

        lock.clear();

        for account in accounts {
            if account.balance() > 0.0 {
                lock.entry(account.r#type()).or_default().push(account);
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<AccountType, Vec<Account>>> {
        self.0.lock().unwrap()
    }
}

impl fmt::Display for Accounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = String::new();

        let lock = self.lock();

        if !lock.is_empty() {
            const TYPES: [AccountType; 4] = [
                AccountType::Main,
                AccountType::Trade,
                AccountType::Contract,
                AccountType::Margin,
            ];

            for r#type in TYPES {
                if let Some(accounts) = lock.get(&r#type) {
                    text.push_str(&format!("<b>{type} Account</b>\n\n"));

                    for account in accounts {
                        text.push_str(&account.to_string());
                    }
                }
            }
        }

        write!(f, "{text}")
    }
}

impl Poller for Accounts {
    async fn poll(&self, client: &Client) {
        match Request::get(ACCOUNTS).send(client).await {
            Ok(accounts) => self.set(accounts),
            Err(e) => error!("{e}"),
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    id: String,
    currency: String,
    r#type: AccountType,
    balance: String,
    available: String,
    holds: String,
}

impl Account {
    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn r#type(&self) -> AccountType {
        self.r#type
    }

    pub fn balance(&self) -> f32 {
        self.balance.parse().unwrap()
    }

    pub fn available(&self) -> f64 {
        self.available.parse().unwrap()
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.r#type {
            AccountType::Main => write!(
                f,
                "<b>Coin:</b> {}\n<b>Total:</b> {}\n\n",
                self.currency, self.balance
            ),
            _ => write!(
                f,
                "<b>Coin:</b> {}\n<b>Total:</b> {}\n<b>Available:</b> {}\n<b>In orders:</b> {}\n\n",
                self.currency, self.balance, self.available, self.holds
            ),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all(serialize = "SCREAMING_SNAKE_CASE", deserialize = "snake_case"))]
pub enum AccountType {
    #[default]
    Main,
    Trade,
    Contract,
    Margin,
    Isolated,
    MarginV2,
    IsolatedV2,
    Option,
}

impl FromStr for AccountType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Funding" => Ok(AccountType::Main),
            "Trading" => Ok(AccountType::Trade),
            "Futures" => Ok(AccountType::Contract),
            "Margin" => Ok(AccountType::Margin),
            "Isolated" => Ok(AccountType::Isolated),
            "Margin V2" => Ok(AccountType::MarginV2),
            "Isolated V2" => Ok(AccountType::IsolatedV2),
            "Option" => Ok(AccountType::Option),
            _ => Err(()),
        }
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Main => write!(f, "Funding"),
            Self::Trade => write!(f, "Trading"),
            Self::Contract => write!(f, "Futures"),
            Self::Margin => write!(f, "Margin"),
            Self::Isolated => write!(f, "Isolated"),
            Self::MarginV2 => write!(f, "Margin V2"),
            Self::IsolatedV2 => write!(f, "Isolated V2"),
            Self::Option => write!(f, "Option"),
        }
    }
}
