use std::vec;
use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::{EditMessageTextSetters as _, SendMessageSetters as _},
    prelude::{Dialogue, Requester as _},
    types::{CallbackQuery, InlineKeyboardMarkup, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    strategies::strategy::{Condition, Strategy},
    telegram::{
        callback_query::{cancel, wrong_button},
        constants::{ABOVE, BELOW, CANCEL},
        keyboard::KeyboardMarkupBuilder as _,
        ConditionState::{Receive, ReceiveValue},
        State,
        StrategyState::Condition as ConditionState,
    },
};

pub async fn edit(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategy: Strategy,
) -> Result<(), RequestError> {
    if let Some(msg) = &query.message {
        let inline_keyboard = [vec![ABOVE, BELOW], vec![CANCEL]];

        bot.send_message(msg.chat().id, "Choose condition:")
            .reply_markup(InlineKeyboardMarkup::from_str_items(inline_keyboard))
            .await?;

        let _ = dialogue
            .update(State::Strategy(ConditionState(Receive { strategy })))
            .await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}

pub async fn receive(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategy: Strategy,
) -> Result<(), RequestError> {
    if let Some((data, msg)) = query.data.as_ref().zip(query.message.as_ref()) {
        let condition = match data.as_str() {
            ABOVE => Condition::GreaterThan(Default::default()),
            BELOW => Condition::LessThan(Default::default()),
            CANCEL => return cancel(bot, query, dialogue).await,
            _ => return wrong_button(bot, query).await,
        };

        bot.edit_message_text(msg.chat().id, msg.id(), format!("Condition: <b>{data}</b>"))
            .parse_mode(Html)
            .await?;

        bot.send_message(msg.chat().id, "Enter value:")
            .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
            .await?;

        let _ = dialogue
            .update(State::Strategy(ConditionState(ReceiveValue {
                strategy,
                condition,
            })))
            .await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}
