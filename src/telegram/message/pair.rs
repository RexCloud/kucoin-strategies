use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendMessageSetters as _,
    prelude::{Dialogue, Requester as _},
    types::{Message, MessageId, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    kucoin::KuCoin,
    telegram::{keyboard, State},
};

pub async fn receive_symbol(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    kucoin: KuCoin,
) -> Result<(), RequestError> {
    bot.edit_message_reply_markup(msg.chat.id, MessageId(msg.id.0 - 1))
        .await?;

    let symbol = msg.text().unwrap_or_default().to_uppercase();

    let maybe_text = kucoin
        .spot()
        .tickers()
        .get(&symbol, true)
        .map(|ticker| ticker.to_string());

    match maybe_text {
        Some(text) => {
            bot.send_message(msg.chat.id, text).parse_mode(Html).await?;

            let _ = dialogue.reset().await;
        }
        None => {
            bot.send_message(msg.chat.id, "Pair not found\n\nEnter pair name:")
                .reply_markup(keyboard::recent_pairs(kucoin.spot()))
                .await?;
        }
    }

    Ok(())
}
