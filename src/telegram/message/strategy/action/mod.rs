use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendMessageSetters as _,
    prelude::{Dialogue, Requester as _},
    types::{InlineKeyboardMarkup, Message, MessageId, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    kucoin::{trading::spot::order::Type, KuCoin},
    strategies::{
        strategy::{Action, ActionKind, Strategy},
        Strategies,
    },
    telegram::{
        constants::CANCEL,
        keyboard::{self, KeyboardMarkupBuilder as _},
        ActionState::{
            ReceiveLendInterestRate, ReceiveOrderPrice, ReceivePercentage, ReceiveTransferFrom,
        },
        State,
        StrategyState::Action as ActionState,
    },
};

pub mod lend;
pub mod order;
pub mod transfer;

pub async fn receive_symbol(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    kucoin: KuCoin,
    (strategy, mut action): (Strategy, Action),
) -> Result<(), RequestError> {
    bot.edit_message_reply_markup(msg.chat.id, MessageId(msg.id.0 - 1))
        .await?;

    let symbol = msg.text().unwrap_or_default().to_uppercase();

    let maybe_error_text = match action.kind() {
        ActionKind::SpotOrder { .. } => kucoin
            .spot()
            .tickers()
            .get(&symbol, false)
            .is_none()
            .then_some("Pair not found\n\nEnter pair name:"),
        ActionKind::Lend { .. } | ActionKind::Redeem => kucoin
            .lending()
            .currencies()
            .get(&symbol, false)
            .is_none()
            .then_some("Currency not found\n\nEnter currency ticker:"),
        ActionKind::Transfer { .. } => None,
    };

    match maybe_error_text {
        Some(text) => {
            bot.send_message(msg.chat.id, text)
                .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
                .await?;
        }
        None => {
            *action.symbol_mut() = symbol;

            let (text, markup, state) = match action.kind() {
                ActionKind::SpotOrder {
                    r#type: Type::Limit,
                    ..
                } => (
                    "Enter price:",
                    InlineKeyboardMarkup::from_str_items([[CANCEL]]),
                    ReceiveOrderPrice { strategy, action },
                ),
                ActionKind::Lend { .. } => (
                    "Enter Min lending APY:",
                    InlineKeyboardMarkup::from_str_items([[CANCEL]]),
                    ReceiveLendInterestRate { strategy, action },
                ),
                ActionKind::Transfer { .. } => (
                    "Choose FROM:",
                    keyboard::choose_account_type(None),
                    ReceiveTransferFrom { strategy, action },
                ),
                _ => (
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
    }

    Ok(())
}

pub async fn receive_percentage(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategies: Strategies,
    (mut strategy, mut action): (Strategy, Action),
) -> Result<(), RequestError> {
    bot.edit_message_reply_markup(msg.chat.id, MessageId(msg.id.0 - 1))
        .await?;

    match msg
        .text()
        .unwrap_or_default()
        .parse()
        .ok()
        .and_then(|value| (1..=100).contains(&value).then_some(value))
    {
        Some(percentage) => {
            *action.percentage_mut() = percentage;
            *action.skip_mut() = false;

            strategy.actions_mut().add(action);

            bot.send_message(msg.chat.id, strategy.to_string())
                .reply_markup(keyboard::edit_strategy())
                .parse_mode(Html)
                .await?;

            strategies.add(strategy);

            let _ = dialogue.reset().await;
        }
        None => {
            bot.send_message(
                msg.chat.id,
                "Value parse error\n\nEnter percentage between 1-100:",
            )
            .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
            .await?;
        }
    }

    Ok(())
}
