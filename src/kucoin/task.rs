use reqwest::Client;
use std::{future::Future, time::Duration};
use tokio::time::interval;

pub trait Poller {
    fn poller(self, client: Client, period: Duration) -> impl Spawnable
    where
        Self: Send + Sized + 'static,
    {
        async move {
            let mut interval = interval(period);
            loop {
                interval.tick().await;
                self.poll(&client).await;
            }
        }
    }

    fn poll(&self, client: &Client) -> impl Future<Output = ()> + Send;
}

pub trait Spawnable {
    fn spawn(self);
}

impl<T> Spawnable for T
where
    T: Future<Output = ()> + Send + 'static,
{
    fn spawn(self) {
        tokio::spawn(self);
    }
}
