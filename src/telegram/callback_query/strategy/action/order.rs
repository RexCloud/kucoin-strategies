use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendMessageSetters as _,
    prelude::{Dialogue, Requester as _},
    types::{CallbackQuery, InlineKeyboardMarkup},
    Bot, RequestError,
};

use crate::{
    kucoin::trading::spot::order::Type,
    strategies::strategy::{Action, ActionKind, Strategy},
    telegram::{
        callback_query::{cancel, wrong_button},
        constants::{CANCEL, LIMIT, MARKET},
        keyboard::KeyboardMarkupBuilder as _,
        ActionState::ReceiveSymbol,
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
        if let ActionKind::SpotOrder { r#type, .. } = action.kind_mut() {
            *r#type = match data.as_str() {
                LIMIT => Type::Limit,
                MARKET => Type::Market,
                CANCEL => return cancel(bot, query, dialogue).await,
                _ => return wrong_button(bot, query).await,
            };

            bot.send_message(msg.chat().id, "Enter spot trading pair name:")
                .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
                .await?;

            let _ = dialogue
                .update(State::Strategy(ActionState(ReceiveSymbol {
                    strategy,
                    action,
                })))
                .await;
        }
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}
