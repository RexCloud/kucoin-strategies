use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendMessageSetters as _,
    prelude::{Dialogue, Requester as _},
    types::{InlineKeyboardMarkup, Message, MessageId, ParseMode::Html},
    Bot, RequestError,
};

use crate::{
    kucoin::KuCoin,
    strategies::{
        strategy::{Product, Strategy},
        Strategies,
    },
    telegram::{
        constants::CANCEL,
        keyboard::{self, KeyboardMarkupBuilder as _},
        ProductState::ReceiveBalanceAccountType,
        State,
        StrategyState::Product as ProductState,
    },
};

pub async fn receive_symbol(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    kucoin: KuCoin,
    strategies: Strategies,
    (mut strategy, product): (Strategy, Product),
) -> Result<(), RequestError> {
    bot.edit_message_reply_markup(msg.chat.id, MessageId(msg.id.0 - 1))
        .await?;

    let symbol = msg.text().unwrap_or_default().to_uppercase();

    let maybe_product = match product {
        Product::SpotTradingPair(_) => kucoin
            .spot()
            .tickers()
            .get(&symbol, false)
            .map(|_| Product::SpotTradingPair(symbol)),
        Product::LendingCurrency(_) => kucoin
            .lending()
            .currencies()
            .get(&symbol, false)
            .map(|_| Product::LendingCurrency(symbol)),
        Product::BalanceCurrency { r#type, .. } => Some(Product::BalanceCurrency {
            r#type,
            currency: symbol,
        }),
    };

    match maybe_product {
        Some(product) => match &product {
            Product::BalanceCurrency { .. } => {
                bot.send_message(msg.chat.id, "Choose account:")
                    .reply_markup(keyboard::choose_account_type(None))
                    .await?;

                let _ = dialogue
                    .update(State::Strategy(ProductState(ReceiveBalanceAccountType {
                        strategy,
                        product,
                    })))
                    .await;
            }
            _ => {
                strategy.set_product(product);

                bot.send_message(msg.chat.id, strategy.to_string())
                    .reply_markup(keyboard::edit_strategy())
                    .parse_mode(Html)
                    .await?;

                strategies.add(strategy);

                let _ = dialogue.reset().await;
            }
        },
        None => {
            let text = match product {
                Product::SpotTradingPair(_) => "Pair not found\n\nEnter pair name:",
                _ => "Currency not found\n\nEnter currency ticker:",
            };

            bot.send_message(msg.chat.id, text)
                .reply_markup(InlineKeyboardMarkup::from_str_items([[CANCEL]]))
                .await?;
        }
    };

    Ok(())
}
