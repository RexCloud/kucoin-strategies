use teloxide::Bot;
use tracing_subscriber::FmtSubscriber;

use kucoin_strategies::{kucoin::KuCoin, strategies::Strategies, telegram};

#[tokio::main]
async fn main() {
    FmtSubscriber::builder().compact().init();

    let bot = Bot::new(env!("BOT_TOKEN"));
    let kucoin: KuCoin = Default::default();
    let strategies: Strategies = Default::default();

    kucoin.clone().run(bot.clone());
    strategies.clone().run(bot.clone(), kucoin.clone());
    telegram::run(bot, kucoin, strategies).await;
}
