use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendMessageSetters as _,
    prelude::{Dialogue, Requester as _},
    types::{InlineKeyboardMarkup, Message, MessageId, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    strategies::{strategy::Strategy, Strategies},
    telegram::{
        constants::CANCEL,
        keyboard::{self, KeyboardMarkupBuilder as _},
        State,
    },
};

pub mod action;
pub mod condition;
pub mod product;

pub async fn receive_name(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategies: Strategies,
    maybe_strategy: Option<Strategy>,
) -> Result<(), RequestError> {
    if let Some(name) = msg.text().map(|text| text.to_string()) {
        bot.edit_message_reply_markup(msg.chat.id, MessageId(msg.id.0 - 1))
            .await?;

        let is_unique = strategies.get(&name).is_none();

        match is_unique {
            true => {
                let strategy =
                    match maybe_strategy.and_then(|strategy| strategies.remove(strategy.name())) {
                        Some(mut strategy) => {
                            strategy.set_name(name);
                            strategy
                        }
                        None => Strategy::new(name),
                    };

                bot.send_message(msg.chat.id, strategy.to_string())
                    .reply_markup(keyboard::edit_strategy())
                    .parse_mode(Html)
                    .await?;

                strategies.add(strategy);

                let _ = dialogue.reset().await;
            }
            false => {
                bot.send_message(
                    msg.chat.id,
                    "Strategy with the same name already exists\n\nEnter strategy name:",
                )
                .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
                .await?;
            }
        }
    }

    Ok(())
}
