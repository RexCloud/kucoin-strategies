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

pub async fn receive_interest_rate(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    kucoin: KuCoin,
    (strategy, mut action): (Strategy, Action),
) -> Result<(), RequestError> {
    bot.edit_message_reply_markup(msg.chat.id, MessageId(msg.id.0 - 1))
        .await?;

    let (min_interest_rate, max_interest_rate, increment) = kucoin
        .lending()
        .currencies()
        .get(action.symbol(), false)
        .map(|currency| {
            (
                currency.min_interest_rate(),
                currency.max_interest_rate(),
                currency.interest_increment(),
            )
        })
        .unwrap_or_default();

    let mut maybe_state = None;

    let text = match msg
        .text()
        .unwrap_or_default()
        .parse()
        .ok()
        .filter(|value| (min_interest_rate..=max_interest_rate).contains(value))
    {
        Some(value) if (value / increment).fract() == 0.0 => {
            if let ActionKind::Lend { interest_rate } = action.kind_mut() {
                *interest_rate = value;
            }

            maybe_state = Some(State::Strategy(ActionState(ReceivePercentage {
                strategy,
                action,
            })));

            "Enter percentage:"
        }
        Some(_) => &format!(
            "Value parse error\n\nEnter Min Lending APY with increment of {}",
            increment
        ),
        None => &format!(
            "Value parse error\n\nEnter Min Lending APY between {}-{}:",
            min_interest_rate, max_interest_rate
        ),
    };

    bot.send_message(msg.chat.id, text)
        .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
        .await?;

    if let Some(state) = maybe_state {
        let _ = dialogue.update(state).await;
    }

    Ok(())
}
