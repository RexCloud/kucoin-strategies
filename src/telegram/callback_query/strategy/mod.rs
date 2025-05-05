use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::{
        AnswerCallbackQuerySetters as _, EditMessageTextSetters as _, SendMessageSetters as _,
    },
    prelude::{Dialogue, Requester as _},
    types::{CallbackQuery, InlineKeyboardMarkup, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    strategies::{strategy::Strategy, Strategies},
    telegram::{
        callback_query::wrong_button,
        constants::{CANCEL, NO, YES},
        keyboard::{self, KeyboardMarkupBuilder as _},
        State,
        StrategyState::{ReceiveDeleteConfirm, ReceiveName},
    },
};

pub mod action;
pub mod condition;
pub mod product;

pub async fn create(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
) -> Result<(), RequestError> {
    if let Some(msg) = query.message {
        bot.send_message(msg.chat().id, "Enter strategy name:")
            .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
            .await?;

        let _ = dialogue
            .update(State::Strategy(ReceiveName {
                maybe_strategy: None,
            }))
            .await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}

pub async fn edit_name(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategy: Strategy,
) -> Result<(), RequestError> {
    if let Some(msg) = &query.message {
        bot.edit_message_reply_markup(msg.chat().id, msg.id())
            .await?;

        bot.send_message(msg.chat().id, "Enter strategy name:")
            .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
            .await?;

        let _ = dialogue
            .update(State::Strategy(ReceiveName {
                maybe_strategy: Some(strategy),
            }))
            .await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}

pub async fn delete(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategy: Strategy,
) -> Result<(), RequestError> {
    if let Some(msg) = &query.message {
        bot.edit_message_text(
            msg.chat().id,
            msg.id(),
            format!("{strategy}\n\nAre you sure you want to delete this strategy?"),
        )
        .reply_markup(InlineKeyboardMarkup::from_str_items([[YES, NO]]))
        .parse_mode(Html)
        .await?;

        let _ = dialogue
            .update(State::Strategy(ReceiveDeleteConfirm { strategy }))
            .await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}

pub async fn back_to_strategies(
    bot: Bot,
    query: CallbackQuery,
    strategies: Strategies,
) -> Result<(), RequestError> {
    if let Some(msg) = &query.message {
        bot.edit_message_text(msg.chat().id, msg.id(), strategies.to_string())
            .reply_markup(keyboard::strategies(&strategies))
            .await?;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}

pub async fn edit_by_name(
    bot: Bot,
    query: CallbackQuery,
    strategy: Strategy,
) -> Result<(), RequestError> {
    if let Some(msg) = query.message {
        bot.edit_message_text(msg.chat().id, msg.id(), strategy.to_string())
            .reply_markup(keyboard::edit_strategy())
            .parse_mode(Html)
            .await?;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}

pub async fn receive_delete_confirm(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategies: Strategies,
    strategy: Strategy,
) -> Result<(), RequestError> {
    if let Some((data, msg)) = query.data.as_ref().zip(query.message.as_ref()) {
        match data.as_str() {
            YES => {
                strategies.remove(strategy.name());

                bot.edit_message_text(msg.chat().id, msg.id(), strategies.to_string())
                    .reply_markup(keyboard::strategies(&strategies))
                    .await?;

                bot.answer_callback_query(query.id)
                    .text("Strategy has been deleted")
                    .await?;
            }
            NO => {
                bot.edit_message_text(msg.chat().id, msg.id(), strategy.to_string())
                    .reply_markup(keyboard::edit_strategy())
                    .parse_mode(Html)
                    .await?;
            }
            _ => return wrong_button(bot, query).await,
        }

        let _ = dialogue.reset().await;
    }

    Ok(())
}
