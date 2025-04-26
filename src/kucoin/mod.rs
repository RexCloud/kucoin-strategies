use reqwest::{header::HeaderMap, Client, ClientBuilder};
use std::{collections::HashMap, time::Duration};

mod constants;

pub mod account;
pub use account::Accounts;

pub mod lending;
pub use lending::Lending;

pub mod trading;
pub use trading::SpotTrading;

pub mod request;
pub use request::Request;

pub mod response;
pub use response::Response;

pub mod task;
use task::{Poller as _, Spawnable as _};

#[derive(Debug, Clone)]
pub struct KuCoin {
    accounts: Accounts,
    lending: Lending,
    spot: SpotTrading,
    client: Client,
}

impl Default for KuCoin {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("KC-API-KEY", env!("API_KEY").parse().unwrap());
        headers.insert(
            "KC-API-KEY-VERSION",
            env!("API_KEY_VERSION").parse().unwrap(),
        );

        KuCoin {
            accounts: Default::default(),
            lending: Default::default(),
            spot: Default::default(),
            client: ClientBuilder::new()
                .default_headers(headers)
                .build()
                .unwrap(),
        }
    }
}

impl KuCoin {
    pub fn accounts(&self) -> &Accounts {
        &self.accounts
    }

    pub fn lending(&self) -> &Lending {
        &self.lending
    }

    pub fn spot(&self) -> &SpotTrading {
        &self.spot
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn run(self) {
        self.accounts()
            .clone()
            .poller(self.client().clone(), Duration::from_secs(15))
            .spawn();

        self.lending()
            .currencies_ref()
            .clone()
            .poller(self.client().clone(), Duration::from_secs(60))
            .spawn();

        self.lending()
            .orders_ref()
            .clone()
            .poller(self.client().clone(), Duration::from_secs(20))
            .spawn();

        self.spot()
            .currencies_ref()
            .clone()
            .poller(self.client().clone(), Duration::from_secs(60))
            .spawn();

        self.spot()
            .symbols_ref()
            .clone()
            .poller(self.client().clone(), Duration::from_secs(60))
            .spawn();

        self.spot()
            .tickers_ref()
            .clone()
            .poller(self.client().clone(), Duration::from_secs(5))
            .spawn();
    }
}

#[derive(Debug)]
pub struct WithRecent<T> {
    inner: HashMap<String, T>,
    recent: Vec<String>,
}

impl<T> WithRecent<T> {
    pub fn get(&mut self, k: &str, add_to_recent: bool) -> Option<&T> {
        self.inner.get(k).inspect(|_| {
            if add_to_recent {
                self.recent.retain(|s| *s != k);

                self.recent.push(k.to_string());

                if self.recent.len() > 4 {
                    self.recent.remove(0);
                }
            }
        })
    }

    pub fn recent(&self) -> Vec<String> {
        self.recent.iter().rev().cloned().collect()
    }
}

impl<T> Default for WithRecent<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            recent: Default::default(),
        }
    }
}
