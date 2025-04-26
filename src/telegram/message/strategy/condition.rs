use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendMessageSetters as _,
    prelude::{Dialogue, Requester as _},
    types::{InlineKeyboardMarkup, Message, MessageId, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    strategies::{
        strategy::{Condition, Strategy},
        Strategies,
    },
    telegram::{
        constants::CANCEL,
        keyboard::{self, KeyboardMarkupBuilder as _},
        State,
    },
};

pub async fn receive_value(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategies: Strategies,
    (mut strategy, condition): (Strategy, Condition),
) -> Result<(), RequestError> {
    bot.edit_message_reply_markup(msg.chat.id, MessageId(msg.id.0 - 1))
        .await?;

    match msg.text().map(|text| text.parse()) {
        Some(Ok(value)) => {
            let condition = match condition {
                Condition::GreaterThan(_) => Condition::GreaterThan(value),
                Condition::LessThan(_) => Condition::LessThan(value),
            };

            strategy.set_condition(condition);

            bot.send_message(msg.chat.id, strategy.to_string())
                .reply_markup(keyboard::edit_strategy())
                .parse_mode(Html)
                .await?;

            strategies.add(strategy);

            let _ = dialogue.reset().await;
        }
        _ => {
            bot.send_message(msg.chat.id, "Value parse error.\n\nEnter value:")
                .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
                .await?;
        }
    }

    Ok(())
}
