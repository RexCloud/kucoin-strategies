use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendMessageSetters as _,
    prelude::{Dialogue, Requester as _},
    types::{InlineKeyboardMarkup, Message, MessageId},
    Bot, RequestError,
};

use crate::{
    kucoin::KuCoin,
    strategies::strategy::{Action, ActionKind, Strategy},
    telegram::{
        constants::CANCEL, keyboard::KeyboardMarkupBuilder as _, ActionState::ReceivePercentage,
        State, StrategyState::Action as ActionState,
    },
};

pub async fn receive_price(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    kucoin: KuCoin,
    (strategy, mut action): (Strategy, Action),
) -> Result<(), RequestError> {
    bot.edit_message_reply_markup(msg.chat.id, MessageId(msg.id.0 - 1))
        .await?;

    let mut maybe_state = None;

    let text = match msg
        .text()
        .unwrap_or_default()
        .parse()
        .ok()
        .filter(|value: &f64| *value > 0.0)
    {
        Some(value) => {
            let maybe_increment = kucoin
                .spot()
                .symbols()
                .get(action.symbol())
                .map(|symbol| symbol.price_increment());

            match maybe_increment {
                Some(increment) if (value / increment).fract() == 0.0 => {
                    if let ActionKind::SpotOrder { price, .. } = action.kind_mut() {
                        *price = Some(value);
                    }

                    maybe_state = Some(State::Strategy(ActionState(ReceivePercentage {
                        strategy,
                        action,
                    })));

                    "Enter percentage:"
                }
                Some(increment) => {
                    &format!("Value parse error\n\nEnter price with increment of {increment}:")
                }
                None => "Internal error\n\nEnter price:",
            }
        }
        None => "Value parse error\n\nEnter price:",
    };

    bot.send_message(msg.chat.id, text)
        .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
        .await?;

    if let Some(state) = maybe_state {
        let _ = dialogue.update(state).await;
    }

    Ok(())
}
