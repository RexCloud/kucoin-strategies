use reqwest::Client;
use std::{
    collections::HashMap,
    fmt,
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};
use tracing::error;

use crate::kucoin::{
    constants::{LENDING_CURRENCIES, LENDING_ORDERS},
    response::Paginated,
    task::Poller,
    Request, WithRecent,
};

mod currency;
pub use currency::Currency;

mod lend;
pub use lend::Lend;

mod order;
pub use order::Order;

mod redeem;
pub use redeem::Redeem;

#[derive(Debug, Default, Clone)]
pub struct Lending {
    currencies: Arc<Mutex<Currencies>>,
    orders: Arc<Mutex<Orders>>,
}

impl Lending {
    pub fn currencies_ref(&self) -> &Arc<Mutex<Currencies>> {
        &self.currencies
    }

    pub fn currencies(&self) -> MutexGuard<'_, Currencies> {
        self.currencies_ref().lock().unwrap()
    }

    pub fn orders_ref(&self) -> &Arc<Mutex<Orders>> {
        &self.orders
    }

    pub fn orders(&self) -> MutexGuard<'_, Orders> {
        self.orders_ref().lock().unwrap()
    }
}

impl fmt::Display for Lending {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = String::new();

        let orders = self.orders();

        if !orders.is_empty() {
            text.push_str("<b>Lending</b>\n\n");

            for order in orders.values() {
                text.push_str(&order.to_string());
            }
        }

        write!(f, "{text}")
    }
}

pub type Currencies = WithRecent<Currency>;

impl Poller for Arc<Mutex<Currencies>> {
    async fn poll(&self, client: &Client) {
        match Request::get(LENDING_CURRENCIES)
            .send::<Vec<Currency>>(client)
            .await
        {
            Ok(currencies) => {
                self.lock().unwrap().inner = currencies
                    .into_iter()
                    .map(|currency| (currency.currency().to_string(), currency))
                    .collect()
            }
            Err(e) => error!("{e}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Orders(HashMap<String, Order>);

impl Deref for Orders {
    type Target = HashMap<String, Order>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Poller for Arc<Mutex<Orders>> {
    async fn poll(&self, client: &Client) {
        match Request::get(LENDING_ORDERS)
            .send::<Paginated<Order>>(client)
            .await
        {
            Ok(paginated) => {
                self.lock().unwrap().0 = paginated
                    .into_iter()
                    .map(|order| (order.currency().to_string(), order))
                    .collect()
            }
            Err(e) => error!("{e}"),
        };
    }
}
