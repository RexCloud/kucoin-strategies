use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::AnswerCallbackQuerySetters as _,
    prelude::{Dialogue, Requester as _},
    types::{CallbackQuery, MaybeInaccessibleMessage},
    Bot, RequestError,
};

use crate::{
    strategies::{strategy::Strategy, Strategies},
    telegram::{
        constants::{
            BACK_TO_STRATEGIES, CANCEL, CREATE_STRATEGY, DELETE_STRATEGY, EDIT_ACTIONS,
            EDIT_CONDITION, EDIT_NAME, EDIT_PRODUCT,
        },
        State,
    },
};

pub mod lending;
pub mod pair;
pub mod strategy;
use strategy::{action, back_to_strategies, condition, product};

pub async fn handler(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategies: Strategies,
) -> Result<(), RequestError> {
    if let Some((data, msg)) = query.data.as_ref().zip(query.message.as_ref()) {
        match dialogue.get().await.unwrap().unwrap() {
            State::Default => match data.as_str() {
                CREATE_STRATEGY => return strategy::create(bot, query, dialogue).await,
                EDIT_NAME | EDIT_PRODUCT | EDIT_CONDITION | EDIT_ACTIONS | DELETE_STRATEGY => {
                    match parse_strategy(msg, &strategies) {
                        Some(strategy) => match data.as_str() {
                            EDIT_NAME => {
                                return strategy::edit_name(bot, query, dialogue, strategy).await
                            }
                            EDIT_PRODUCT => {
                                return product::edit(bot, query, dialogue, strategy).await
                            }
                            EDIT_CONDITION => {
                                return condition::edit(bot, query, dialogue, strategy).await
                            }
                            EDIT_ACTIONS => {
                                return action::edit(bot, query, dialogue, strategy).await
                            }
                            _ => return strategy::delete(bot, query, dialogue, strategy).await,
                        },
                        None => return wrong_button(bot, query).await,
                    }
                }
                BACK_TO_STRATEGIES => return back_to_strategies(bot, query, strategies).await,
                _ => return strategy::edit_by_name(bot, query, strategies).await,
            },
            _ => match data.as_str() {
                CANCEL => return cancel(bot, query, dialogue).await,
                _ => return wrong_button(bot, query).await,
            },
        }
    }

    Ok(())
}

async fn cancel(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
) -> Result<(), RequestError> {
    bot.answer_callback_query(query.id)
        .text("Operation canceled")
        .await?;

    if let Some(msg) = query.message {
        bot.delete_message(msg.chat().id, msg.id()).await?;
    }

    let _ = dialogue.reset().await;

    Ok(())
}

async fn wrong_button(bot: Bot, query: CallbackQuery) -> Result<(), RequestError> {
    bot.answer_callback_query(query.id)
        .text("Wrong button selected")
        .await?;

    Ok(())
}

fn parse_strategy(msg: &MaybeInaccessibleMessage, strategies: &Strategies) -> Option<Strategy> {
    msg.regular_message()
        .and_then(|msg| msg.text())
        .and_then(|text| text.lines().next())
        .and_then(|first_line| first_line.get(6..))
        .and_then(|name| strategies.get(name))
}
