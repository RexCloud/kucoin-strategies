use reqwest::Client;
use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};
use teloxide::{
    payloads::SendMessageSetters as _, prelude::Requester as _, types::ParseMode::Html, Bot,
};
use tracing::error;

use crate::kucoin::{
    response::Order,
    task::{Poller, Spawnable as _},
    KuCoin,
};

pub mod strategy;
use strategy::Strategy;

#[derive(Debug, Default, Clone)]
pub struct Strategies(Arc<Mutex<HashMap<String, Strategy>>>);

impl Strategies {
    pub fn get(&self, name: &str) -> Option<Strategy> {
        self.lock().get(name).cloned()
    }

    pub fn names(&self) -> Vec<String> {
        self.lock().keys().cloned().collect()
    }

    pub fn add(&self, strategy: Strategy) {
        self.lock().insert(strategy.name().to_string(), strategy);
    }

    pub fn remove(&self, name: &str) -> Option<Strategy> {
        self.lock().remove(name)
    }

    pub fn run(self, bot: Bot, kucoin: KuCoin) {
        let client = kucoin.client().clone();

        (self, bot, kucoin)
            .poller(client, Duration::from_secs(1))
            .spawn();
    }

    fn executable(&self, kucoin: &KuCoin) -> Option<Strategy> {
        self.lock()
            .values()
            .find(|strategy| !strategy.actions().is_empty() && strategy.can_execute(kucoin))
            .cloned()
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, Strategy>> {
        self.0.lock().unwrap()
    }
}

impl fmt::Display for Strategies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.lock().len();

        let ending = match len % 10 != 1 {
            true => "ies",
            false => "y",
        };

        write!(f, "Found {len} strateg{ending}")
    }
}

impl Poller for (Strategies, Bot, KuCoin) {
    async fn poll(&self, client: &Client) {
        let strategies = &self.0;
        let bot = &self.1;
        let kucoin = &self.2;

        if let Some(strategy) = strategies.executable(kucoin) {
            let name = strategy.name().to_string();
            let mut strategy = strategy;
            let mut remaining = strategy.actions().len();

            while let Some((action, request)) = strategy.actions_mut().executable(kucoin) {
                let mut text = format!(
                    "<b>Strategy:</b> {}\n\n<b>Action:</b> {}\n\n<b>Execution status:</b> ",
                    name, action
                );

                match request.send::<Order>(client).await {
                    Ok(order) => {
                        kucoin.accounts().poll(client).await;

                        if action.percentage() != 100 {
                            *action.skip_mut() = true;
                            strategies.add(strategy.clone());
                        }

                        text.push_str(&format!("✅\n{order}"))
                    }
                    Err(e) => text.push_str(&format!("❌\n{e}")),
                }

                if let Err(e) = bot
                    .send_message(env!("USER_ID").to_string(), text)
                    .parse_mode(Html)
                    .await
                {
                    error!("{e}")
                }

                match remaining {
                    1 => break,
                    _ => remaining -= 1,
                }
            }
        }
    }
}
