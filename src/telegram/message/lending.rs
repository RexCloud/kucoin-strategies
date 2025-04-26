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

pub async fn receive_currency(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    kucoin: KuCoin,
) -> Result<(), RequestError> {
    bot.edit_message_reply_markup(msg.chat.id, MessageId(msg.id.0 - 1))
        .await?;

    let currency = msg.text().unwrap_or_default().to_uppercase();

    let maybe_text = kucoin
        .lending()
        .currencies()
        .get(&currency, true)
        .map(|currency| currency.to_string());

    match maybe_text {
        Some(text) => {
            bot.send_message(msg.chat.id, text).parse_mode(Html).await?;

            let _ = dialogue.reset().await;
        }
        None => {
            bot.send_message(msg.chat.id, "Currency not found\n\nEnter currency name:")
                .reply_markup(keyboard::recent_currencies(kucoin.lending()))
                .await?;
        }
    }

    Ok(())
}
