use std::vec;
use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::{AnswerCallbackQuerySetters, EditMessageTextSetters as _, SendMessageSetters as _},
    prelude::{Dialogue, Requester as _},
    types::{CallbackQuery, InlineKeyboardMarkup, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    strategies::{
        strategy::{Action, ActionKind, Strategy},
        Strategies,
    },
    telegram::{
        callback_query::{cancel, wrong_button},
        constants::{
            ADD_ACTION, BACK_TO_ACTIONS, BUY, CANCEL, DELETE_ACTION, LEND, LIMIT, MARKET,
            MOVE_DOWN, MOVE_UP, NO, REDEEM, SELL, TRANSFER, YES,
        },
        keyboard::{self, KeyboardMarkupBuilder as _},
        ActionState::{
            Receive, ReceiveActionModif, ReceiveActionNumber, ReceiveDeleteConfirm, ReceiveOrder,
            ReceiveSymbol,
        },
        State,
        StrategyState::Action as ActionState,
    },
};

pub mod order;
pub mod transfer;

pub async fn edit(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategy: Strategy,
) -> Result<(), RequestError> {
    if let Some(msg) = &query.message {
        let (text, markup, state) = match strategy.actions().is_empty() {
            true => (
                "Choose action:",
                keyboard::choose_action(),
                Receive { strategy },
            ),
            false => (
                "Choose action number:",
                keyboard::choose_action_number(strategy.actions()),
                ReceiveActionNumber { strategy },
            ),
        };

        bot.send_message(msg.chat().id, text)
            .reply_markup(markup)
            .await?;

        let _ = dialogue.update(State::Strategy(ActionState(state))).await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}

pub async fn receive_number(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategy: Strategy,
) -> Result<(), RequestError> {
    if let Some((data, msg)) = query.data.as_ref().zip(query.message.as_ref()) {
        let (text, markup, state) = match data.as_str() {
            ADD_ACTION => (
                "Choose action:".to_string(),
                keyboard::choose_action(),
                Receive { strategy },
            ),
            CANCEL => return cancel(bot, query, dialogue).await,
            _ => {
                let index = data.parse::<usize>().unwrap() - 1;

                match strategy.actions().get(index) {
                    Some(action) => {
                        let mut inline_keyboard = [vec![], vec![DELETE_ACTION, BACK_TO_ACTIONS]];

                        if index > 0 {
                            inline_keyboard[0].push(MOVE_UP);
                        }

                        if index < strategy.actions().len() - 1 {
                            inline_keyboard[0].push(MOVE_DOWN);
                        }

                        (
                            format!("{}: {}", data, action),
                            InlineKeyboardMarkup::from_str_items(inline_keyboard),
                            ReceiveActionModif { strategy, index },
                        )
                    }
                    _ => return wrong_button(bot, query).await,
                }
            }
        };

        bot.edit_message_text(msg.chat().id, msg.id(), text)
            .reply_markup(markup)
            .await?;

        let _ = dialogue.update(State::Strategy(ActionState(state))).await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}

pub async fn receive_modif(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategies: Strategies,
    (mut strategy, index): (Strategy, usize),
) -> Result<(), RequestError> {
    if let Some((data, msg)) = query.data.as_ref().zip(query.message.as_ref()) {
        let state = match data.as_str() {
            MOVE_UP | MOVE_DOWN => {
                strategy.actions_mut().r#move(index, data == MOVE_UP);

                bot.edit_message_text(msg.chat().id, msg.id(), strategy.to_string())
                    .reply_markup(keyboard::edit_strategy())
                    .parse_mode(Html)
                    .await?;

                strategies.add(strategy);

                Default::default()
            }
            DELETE_ACTION => {
                bot.edit_message_text(
                    msg.chat().id,
                    msg.id(),
                    msg.regular_message()
                        .and_then(|msg| msg.text())
                        .map(|text| text.to_string())
                        .unwrap_or_default()
                        + "\n\nAre you sure you want to delete this action?",
                )
                .reply_markup(InlineKeyboardMarkup::from_str_items([[YES, NO]]))
                .await?;

                State::Strategy(ActionState(ReceiveDeleteConfirm { strategy, index }))
            }
            BACK_TO_ACTIONS => {
                bot.edit_message_text(msg.chat().id, msg.id(), "Choose action number:")
                    .reply_markup(keyboard::choose_action_number(strategy.actions()))
                    .await?;

                State::Strategy(ActionState(ReceiveActionNumber { strategy }))
            }
            _ => return wrong_button(bot, query).await,
        };

        let _ = dialogue.update(state).await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}

pub async fn receive_delete_confirm(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategies: Strategies,
    (mut strategy, index): (Strategy, usize),
) -> Result<(), RequestError> {
    if let Some((data, msg)) = query.data.as_ref().zip(query.message.as_ref()) {
        match data.as_str() {
            YES => {
                strategy.actions_mut().remove(index);

                bot.edit_message_text(msg.chat().id, msg.id(), strategy.to_string())
                    .reply_markup(keyboard::edit_strategy())
                    .parse_mode(Html)
                    .await?;

                strategies.add(strategy);

                let _ = dialogue.reset().await;
            }
            NO => return cancel(bot, query, dialogue).await,
            _ => return wrong_button(bot, query).await,
        }
    }

    bot.answer_callback_query(query.id)
        .text("Action has been deleted")
        .await?;

    Ok(())
}

pub async fn receive(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategy: Strategy,
) -> Result<(), RequestError> {
    if let Some((data, msg)) = query.data.as_ref().zip(query.message.as_ref()) {
        let (text, markup) = match data.as_str() {
            BUY | SELL => (
                "Choose type:",
                InlineKeyboardMarkup::from_str_items([vec![LIMIT, MARKET], vec![CANCEL]]),
            ),
            LEND | REDEEM | TRANSFER => (
                "Enter currency ticker:",
                InlineKeyboardMarkup::from_str_items([[CANCEL]]),
            ),
            CANCEL => return cancel(bot, query, dialogue).await,
            _ => return wrong_button(bot, query).await,
        };

        bot.edit_message_text(msg.chat().id, msg.id(), format!("Action: <b>{data}</b>"))
            .parse_mode(Html)
            .await?;

        bot.send_message(msg.chat().id, text)
            .reply_markup(markup)
            .await?;

        let action = match data.as_str() {
            BUY => Action::buy(),
            SELL => Action::sell(),
            LEND => Action::lend(),
            REDEEM => Action::redeem(),
            _ => Action::transfer(),
        };

        let state = match action.kind() {
            ActionKind::SpotOrder { .. } => ReceiveOrder { strategy, action },
            _ => ReceiveSymbol { strategy, action },
        };

        let _ = dialogue.update(State::Strategy(ActionState(state))).await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}
