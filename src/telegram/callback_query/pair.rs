use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendMessageSetters as _,
    prelude::{Dialogue, Requester as _},
    types::{CallbackQuery, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    kucoin::KuCoin,
    telegram::{
        callback_query::{cancel, wrong_button},
        constants::CANCEL,
        State,
    },
};

pub async fn receive_symbol(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    kucoin: KuCoin,
) -> Result<(), RequestError> {
    if let Some((data, msg)) = query.data.as_ref().zip(query.message.as_ref()) {
        let Some(text) = (match data.as_str() {
            CANCEL => return cancel(bot, query, dialogue).await,
            _ => kucoin
                .spot()
                .tickers()
                .get(data, true)
                .map(|ticker| ticker.to_string()),
        }) else {
            return wrong_button(bot, query).await;
        };

        bot.edit_message_reply_markup(msg.chat().id, msg.id())
            .await?;

        bot.send_message(msg.chat().id, text)
            .parse_mode(Html)
            .await?;

        let _ = dialogue.reset().await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}
