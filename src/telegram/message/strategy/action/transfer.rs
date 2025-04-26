use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendMessageSetters as _,
    prelude::{Dialogue, Requester as _},
    types::{InlineKeyboardMarkup, Message, MessageId},
    Bot, RequestError,
};

use crate::{
    strategies::strategy::{Action, ActionKind, Strategy},
    telegram::{
        constants::CANCEL,
        keyboard::{self, KeyboardMarkupBuilder as _},
        ActionState::{ReceivePercentage, ReceiveTransferFromAccountTag, ReceiveTransferTo},
        State,
        StrategyState::Action as ActionState,
    },
};

pub async fn receive_account_tag(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    (strategy, mut action): (Strategy, Action),
) -> Result<(), RequestError> {
    bot.edit_message_reply_markup(msg.chat.id, MessageId(msg.id.0 - 1))
        .await?;

    let symbol = msg.text().unwrap_or_default().to_uppercase();

    match symbol.contains(action.symbol()) {
        true => {
            let from = matches!(
                dialogue.get().await.unwrap().unwrap(),
                State::Strategy(ActionState(ReceiveTransferFromAccountTag { .. }))
            );

            if let ActionKind::Transfer {
                from_account_tag,
                to_account_tag,
                ..
            } = action.kind_mut()
            {
                match from {
                    true => *from_account_tag = Some(symbol),
                    false => *to_account_tag = Some(symbol),
                }
            }

            let (text, markup, state) = match from {
                true => (
                    "Choose TO:",
                    keyboard::choose_account_type(None),
                    ReceiveTransferTo { strategy, action },
                ),
                false => (
                    "Enter percentage:",
                    InlineKeyboardMarkup::from_str_items([[CANCEL]]),
                    ReceivePercentage { strategy, action },
                ),
            };

            bot.send_message(msg.chat.id, text)
                .reply_markup(markup)
                .await?;

            let _ = dialogue.update(State::Strategy(ActionState(state))).await;
        }
        false => {
            bot.send_message(msg.chat.id, "Value parse error\n\nEnter symbol:")
                .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
                .await?;
        }
    }

    Ok(())
}
