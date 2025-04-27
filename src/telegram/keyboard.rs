use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

use crate::{
    kucoin::{account::AccountType, Lending, SpotTrading},
    strategies::{strategy::Actions, Strategies},
    telegram::constants::{
        ADD_ACTION, BACK_TO_ACTIONS, BACK_TO_STRATEGIES, BUY, CANCEL, CREATE_STRATEGY,
        DELETE_ACTION, DELETE_STRATEGY, EDIT_ACTIONS, EDIT_CONDITION, EDIT_NAME, EDIT_PRODUCT,
        LEND, MOVE_DOWN, MOVE_UP, REDEEM, SELL, TRANSFER,
    },
};

pub trait KeyboardMarkupBuilder {
    fn from_str_items<S, I>(_: I) -> Self
    where
        S: Into<String> + Clone,
        I: IntoIterator,
        I::Item: IntoIterator<Item = S>;
}

impl KeyboardMarkupBuilder for KeyboardMarkup {
    fn from_str_items<S, K>(keyboard: K) -> Self
    where
        S: Into<String> + Clone,
        K: IntoIterator,
        K::Item: IntoIterator<Item = S>,
    {
        Self::new(
            keyboard
                .into_iter()
                .map(|row| row.into_iter().map(|text| KeyboardButton::new(text))),
        )
    }
}

impl KeyboardMarkupBuilder for InlineKeyboardMarkup {
    fn from_str_items<S, I>(inline_keyboard: I) -> Self
    where
        S: Into<String> + Clone,
        I: IntoIterator,
        I::Item: IntoIterator<Item = S>,
    {
        Self::new(inline_keyboard.into_iter().map(|row| {
            row.into_iter()
                .map(|text| InlineKeyboardButton::callback(text.clone(), text))
        }))
    }
}

pub fn strategies(strategies: &Strategies) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::from_str_items([strategies.names(), vec![CREATE_STRATEGY.to_string()]])
}

pub fn edit_strategy() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::from_str_items([
        vec![EDIT_NAME, EDIT_PRODUCT],
        vec![EDIT_CONDITION, EDIT_ACTIONS],
        vec![DELETE_STRATEGY, BACK_TO_STRATEGIES],
    ])
}

pub fn choose_action_number(actions: &Actions) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::from_str_items([
        (1..actions.len() + 1)
            .map(|index| index.to_string())
            .collect(),
        vec![ADD_ACTION.to_string()],
        vec![CANCEL.to_string()],
    ])
}

pub fn edit_action(index: usize, actions: &Actions) -> InlineKeyboardMarkup {
    let mut inline_keyboard = [vec![], vec![DELETE_ACTION, BACK_TO_ACTIONS]];

    if index > 0 {
        inline_keyboard[0].push(MOVE_UP);
    }

    if index < actions.len() - 1 {
        inline_keyboard[0].push(MOVE_DOWN);
    }

    InlineKeyboardMarkup::from_str_items(inline_keyboard)
}

pub fn choose_action() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::from_str_items([
        vec![BUY, SELL],
        vec![LEND, REDEEM],
        vec![TRANSFER],
        vec![CANCEL],
    ])
}

pub fn choose_account_type(skip: Option<&AccountType>) -> InlineKeyboardMarkup {
    let mut inline_keyboard = [
        vec![
            AccountType::Main.to_string(),
            AccountType::Trade.to_string(),
            AccountType::Contract.to_string(),
        ],
        vec![
            AccountType::Margin.to_string(),
            AccountType::Isolated.to_string(),
            AccountType::MarginV2.to_string(),
            AccountType::IsolatedV2.to_string(),
        ],
        vec![AccountType::Option.to_string()],
        vec![CANCEL.to_string()],
    ];

    if let Some(skip) = skip.map(|r#type| r#type.to_string()) {
        for row in inline_keyboard.iter_mut() {
            row.retain(|text| *text != skip);
        }
    }

    InlineKeyboardMarkup::from_str_items(inline_keyboard)
}

pub fn recent_pairs(spot: &SpotTrading) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::from_str_items([spot.tickers().recent(), vec![CANCEL.to_string()]])
}

pub fn recent_currencies(lending: &Lending) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::from_str_items([lending.currencies().recent(), vec![CANCEL.to_string()]])
}
