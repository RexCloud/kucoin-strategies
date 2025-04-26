use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};
use tracing::error;

use crate::kucoin::{
    constants::{SPOT_CURRENCIES, SPOT_SYMBOLS, SPOT_TICKERS},
    task::Poller,
    Request, WithRecent,
};

pub mod order;

mod currency;
use currency::Currency;

mod symbol;
use symbol::Symbol;

mod ticker;
use ticker::Ticker;

#[derive(Debug, Default, Clone)]
pub struct SpotTrading {
    currencies: Arc<Mutex<Currencies>>,
    symbols: Arc<Mutex<Symbols>>,
    tickers: Arc<Mutex<Tickers>>,
}

impl SpotTrading {
    pub fn currencies_ref(&self) -> &Arc<Mutex<Currencies>> {
        &self.currencies
    }
    pub fn currencies(&self) -> MutexGuard<'_, Currencies> {
        self.currencies_ref().lock().unwrap()
    }
    pub fn symbols_ref(&self) -> &Arc<Mutex<Symbols>> {
        &self.symbols
    }

    pub fn symbols(&self) -> MutexGuard<'_, Symbols> {
        self.symbols_ref().lock().unwrap()
    }
    pub fn tickers_ref(&self) -> &Arc<Mutex<Tickers>> {
        &self.tickers
    }

    pub fn tickers(&self) -> MutexGuard<'_, Tickers> {
        self.tickers_ref().lock().unwrap()
    }
}

#[derive(Debug, Default)]
pub struct Currencies(HashMap<String, Currency>);

impl Deref for Currencies {
    type Target = HashMap<String, Currency>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Poller for Arc<Mutex<Currencies>> {
    async fn poll(&self, client: &Client) {
        match Request::get(SPOT_CURRENCIES)
            .send::<Vec<Currency>>(client)
            .await
        {
            Ok(currencies) => {
                self.lock().unwrap().0 = currencies
                    .into_iter()
                    .map(|currency| (currency.currency().to_string(), currency))
                    .collect()
            }
            Err(e) => error!("{e}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Symbols(HashMap<String, Symbol>);

impl Deref for Symbols {
    type Target = HashMap<String, Symbol>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Poller for Arc<Mutex<Symbols>> {
    async fn poll(&self, client: &Client) {
        match Request::get(SPOT_SYMBOLS).send::<Vec<Symbol>>(client).await {
            Ok(symbols) => {
                self.lock().unwrap().0 = symbols
                    .into_iter()
                    .map(|symbol| (symbol.symbol().to_string(), symbol))
                    .collect()
            }
            Err(e) => error!("{e}"),
        }
    }
}

pub type Tickers = WithRecent<Ticker>;

impl Poller for Arc<Mutex<Tickers>> {
    async fn poll(&self, client: &Client) {
        #[derive(Debug, Serialize, Deserialize)]
        struct Response {
            time: u64,
            #[serde(rename = "ticker")]
            tickers: Vec<Ticker>,
        }

        match Request::get(SPOT_TICKERS).send::<Response>(client).await {
            Ok(r) => {
                self.lock().unwrap().inner = r
                    .tickers
                    .into_iter()
                    .map(|ticker| (ticker.symbol().to_string(), ticker))
                    .collect()
            }
            Err(e) => error!("{e}"),
        }
    }
}
