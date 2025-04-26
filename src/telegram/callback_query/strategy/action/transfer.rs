use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::{EditMessageTextSetters as _, SendMessageSetters as _},
    prelude::{Dialogue, Requester as _},
    types::{CallbackQuery, InlineKeyboardMarkup, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    kucoin::account::AccountType::{Isolated, IsolatedV2},
    strategies::strategy::{Action, ActionKind, Strategy},
    telegram::{
        callback_query::{cancel, wrong_button},
        constants::CANCEL,
        keyboard::{self, KeyboardMarkupBuilder as _},
        ActionState::{
            ReceivePercentage, ReceiveTransferFrom, ReceiveTransferFromAccountTag,
            ReceiveTransferTo, ReceiveTransferToAccountTag,
        },
        State,
        StrategyState::Action as ActionState,
    },
};

pub async fn receive(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    (strategy, mut action): (Strategy, Action),
) -> Result<(), RequestError> {
    if let Some((data, msg)) = query.data.as_ref().zip(query.message.as_ref()) {
        if let ActionKind::Transfer { from, to, .. } = action.kind_mut() {
            let Ok(r#type) = (match data.as_str() {
                CANCEL => return cancel(bot, query, dialogue).await,
                _ => data.parse(),
            }) else {
                return wrong_button(bot, query).await;
            };

            let is_from = matches!(
                dialogue.get().await.unwrap().unwrap(),
                State::Strategy(ActionState(ReceiveTransferFrom { .. }))
            );

            match is_from {
                true => *from = r#type,
                false => *to = r#type,
            }

            let text =
                if is_from { "From" } else { "To" }.to_string() + &format!(": <b>{data}</b>");

            bot.edit_message_text(msg.chat().id, msg.id(), text)
                .parse_mode(Html)
                .await?;

            let (text, markup, state) = match r#type {
                Isolated | IsolatedV2 => (
                    "Enter symbol:",
                    InlineKeyboardMarkup::from_str_items([[CANCEL]]),
                    if is_from {
                        ReceiveTransferFromAccountTag { strategy, action }
                    } else {
                        ReceiveTransferToAccountTag { strategy, action }
                    },
                ),
                _ => {
                    if is_from {
                        (
                            "Choose TO:",
                            keyboard::choose_account_type(Some(&r#type)),
                            ReceiveTransferTo { strategy, action },
                        )
                    } else {
                        (
                            "Enter percentage:",
                            InlineKeyboardMarkup::from_str_items([[CANCEL]]),
                            ReceivePercentage { strategy, action },
                        )
                    }
                }
            };

            bot.send_message(msg.chat().id, text)
                .reply_markup(markup)
                .await?;

            let _ = dialogue.update(State::Strategy(ActionState(state))).await;
        }
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}
