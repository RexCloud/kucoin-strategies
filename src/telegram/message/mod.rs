use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendMessageSetters as _,
    prelude::{Dialogue, Requester as _},
    types::{KeyboardMarkup, Message, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    kucoin::KuCoin,
    strategies::Strategies,
    telegram::{
        constants::{BALANCE, LENDING, PAIRS, STRATEGIES},
        keyboard::{self, KeyboardMarkupBuilder as _},
        State,
    },
};

pub mod lending;
pub mod pair;
pub mod strategy;

pub async fn handler(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    kucoin: KuCoin,
    strategies: Strategies,
) -> Result<(), RequestError> {
    if let Some(text) = msg.text() {
        match text {
            STRATEGIES => {
                bot.send_message(msg.chat.id, strategies.to_string())
                    .reply_markup(keyboard::strategies(&strategies))
                    .await?;
            }
            BALANCE => {
                let text = match kucoin.lending().to_string() + &kucoin.accounts().to_string() {
                    text if text.is_empty() => {
                        "Account balance is empty or wasn't fetched yet".to_string()
                    }
                    text => text,
                };

                bot.send_message(msg.chat.id, text).parse_mode(Html).await?;
            }
            PAIRS => {
                bot.send_message(msg.chat.id, "Enter spot trading pair name:")
                    .reply_markup(keyboard::recent_pairs(kucoin.spot()))
                    .await?;

                let _ = dialogue.update(State::ReceivePairSymbol).await;
            }
            LENDING => {
                bot.send_message(msg.chat.id, "Enter currency ticker:")
                    .reply_markup(keyboard::recent_currencies(kucoin.lending()))
                    .await?;

                let _ = dialogue.update(State::ReceiveLendingCurrency).await;
            }
            _ => {
                let text = format!(
                    "<b>{STRATEGIES}</b> — create and view strategies\n\
                    <b>{BALANCE}</b> — show overall balance\n\
                    <b>{PAIRS}</b> — show spot pairs info\n\
                    <b>{LENDING}</b> — show lending currencies info"
                );

                let keyboard = [[STRATEGIES, BALANCE], [PAIRS, LENDING]];

                let markup = KeyboardMarkup::from_str_items(keyboard)
                    .persistent()
                    .resize_keyboard();

                bot.send_message(msg.chat.id, text)
                    .reply_markup(markup)
                    .parse_mode(Html)
                    .await?;
            }
        }
    }

    Ok(())
}
