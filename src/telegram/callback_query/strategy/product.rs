use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::{EditMessageTextSetters as _, SendMessageSetters as _},
    prelude::{Dialogue, Requester as _},
    types::{CallbackQuery, InlineKeyboardMarkup, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    strategies::{
        strategy::{Product, Strategy},
        Strategies,
    },
    telegram::{
        callback_query::{cancel, wrong_button},
        constants::{BALANCE, CANCEL, LENDING, SPOT_TRADING},
        keyboard::{self, KeyboardMarkupBuilder as _},
        ProductState::{Receive, ReceiveSymbol},
        State,
        StrategyState::Product as ProductState,
    },
};

pub async fn edit(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategy: Strategy,
) -> Result<(), RequestError> {
    if let Some(msg) = &query.message {
        let inline_keyboard = [vec![SPOT_TRADING, LENDING, BALANCE], vec![CANCEL]];

        bot.send_message(msg.chat().id, "Choose product:")
            .reply_markup(InlineKeyboardMarkup::from_str_items(inline_keyboard))
            .await?;

        let _ = dialogue
            .update(State::Strategy(ProductState(Receive { strategy })))
            .await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}

pub async fn receive(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategy: Strategy,
) -> Result<(), RequestError> {
    if let Some((data, msg)) = query.data.as_ref().zip(query.message.as_ref()) {
        let text = match data.as_str() {
            SPOT_TRADING => "Enter spot trading pair name:",
            LENDING | BALANCE => "Enter currency ticker:",
            CANCEL => return cancel(bot, query, dialogue).await,
            _ => return wrong_button(bot, query).await,
        };

        bot.edit_message_text(msg.chat().id, msg.id(), format!("Product: <b>{data}</b>"))
            .parse_mode(Html)
            .await?;

        bot.send_message(msg.chat().id, text)
            .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
            .await?;

        let product = match data.as_str() {
            SPOT_TRADING => Product::SpotTradingPair(Default::default()),
            LENDING => Product::LendingCurrency(Default::default()),
            _ => Product::BalanceCurrency {
                r#type: Default::default(),
                currency: Default::default(),
            },
        };

        let _ = dialogue
            .update(State::Strategy(ProductState(ReceiveSymbol {
                strategy,
                product,
            })))
            .await;
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}

pub async fn receive_balance_account_type(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue<State, InMemStorage<State>>,
    strategies: Strategies,
    (mut strategy, product): (Strategy, Product),
) -> Result<(), RequestError> {
    if let Some((data, msg)) = query.data.as_ref().zip(query.message.as_ref()) {
        if let Product::BalanceCurrency { currency, .. } = product {
            let Ok(r#type) = (match data.as_str() {
                CANCEL => return cancel(bot, query, dialogue).await,
                _ => data.parse(),
            }) else {
                return wrong_button(bot, query).await;
            };

            strategy.set_product(Product::BalanceCurrency { r#type, currency });

            bot.edit_message_text(msg.chat().id, msg.id(), strategy.to_string())
                .reply_markup(keyboard::edit_strategy())
                .parse_mode(Html)
                .await?;

            strategies.add(strategy);

            let _ = dialogue.reset().await;
        }
    }

    bot.answer_callback_query(query.id).await?;

    Ok(())
}
