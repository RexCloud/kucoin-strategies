use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateFilterExt as _, UpdateHandler,
    },
    dptree,
    prelude::{Dispatcher, LoggingErrorHandler},
    types::Update,
    update_listeners::Polling,
    Bot, RequestError,
};
use tracing::info;

use crate::{
    kucoin::KuCoin,
    strategies::{
        strategy::{Action, Condition, Product, Strategy},
        Strategies,
    },
};

mod callback_query;
mod constants;
mod keyboard;
mod message;

#[derive(Default, Clone)]
enum State {
    #[default]
    Default,
    Strategy(StrategyState),
    ReceivePairSymbol,
    ReceiveLendingCurrency,
}

#[derive(Clone)]
enum StrategyState {
    ReceiveName { maybe_strategy: Option<Strategy> },
    ReceiveDeleteConfirm { strategy: Strategy },
    Product(ProductState),
    Condition(ConditionState),
    Action(ActionState),
}

#[derive(Clone)]
enum ProductState {
    Receive {
        strategy: Strategy,
    },
    ReceiveSymbol {
        strategy: Strategy,
        product: Product,
    },
    ReceiveBalanceAccountType {
        strategy: Strategy,
        product: Product,
    },
}

#[derive(Clone)]
enum ConditionState {
    Receive {
        strategy: Strategy,
    },
    ReceiveValue {
        strategy: Strategy,
        condition: Condition,
    },
}

#[derive(Clone)]
enum ActionState {
    ReceiveActionNumber { strategy: Strategy },
    ReceiveActionModif { strategy: Strategy, index: usize },
    ReceiveDeleteConfirm { strategy: Strategy, index: usize },
    Receive { strategy: Strategy },
    ReceiveOrder { strategy: Strategy, action: Action },
    ReceiveOrderPrice { strategy: Strategy, action: Action },
    ReceiveSymbol { strategy: Strategy, action: Action },
    ReceiveLendInterestRate { strategy: Strategy, action: Action },
    ReceiveTransferFrom { strategy: Strategy, action: Action },
    ReceiveTransferTo { strategy: Strategy, action: Action },
    ReceiveTransferFromAccountTag { strategy: Strategy, action: Action },
    ReceiveTransferToAccountTag { strategy: Strategy, action: Action },
    ReceivePercentage { strategy: Strategy, action: Action },
}

pub async fn run(bot: Bot, kucoin: KuCoin, strategies: Strategies) {
    let update_listener = Polling::builder(bot.clone()).drop_pending_updates().build();

    let update_listener_error_handler =
        LoggingErrorHandler::with_custom_text("An error from the update listener");

    Dispatcher::builder(bot, schema())
        .enable_ctrlc_handler()
        .dependencies(dptree::deps![
            InMemStorage::<State>::new(),
            kucoin,
            strategies
        ])
        .default_handler(|upd| async move { info!(?upd, "Update from unknown user") })
        .build()
        .dispatch_with_listener(update_listener, update_listener_error_handler)
        .await;
}

fn schema() -> UpdateHandler<RequestError> {
    use dptree::case;

    let message_handler = Update::filter_message()
        .filter(user_id_pred)
        .branch(
            case![State::Strategy(state)]
                .branch(
                    case![StrategyState::ReceiveName { maybe_strategy }]
                        .endpoint(message::strategy::receive_name),
                )
                .branch(
                    case![StrategyState::Product(state)].branch(
                        case![ProductState::ReceiveSymbol { strategy, product }]
                            .endpoint(message::strategy::product::receive_symbol),
                    ),
                )
                .branch(
                    case![StrategyState::Condition(state)].branch(
                        case![ConditionState::ReceiveValue {
                            strategy,
                            condition
                        }]
                        .endpoint(message::strategy::condition::receive_value),
                    ),
                )
                .branch(
                    case![StrategyState::Action(state)]
                        .branch(
                            case![ActionState::ReceiveSymbol { strategy, action }]
                                .endpoint(message::strategy::action::receive_symbol),
                        )
                        .branch(
                            case![ActionState::ReceiveOrderPrice { strategy, action }]
                                .endpoint(message::strategy::action::order::receive_price),
                        )
                        .branch(
                            case![ActionState::ReceiveLendInterestRate { strategy, action }]
                                .endpoint(message::strategy::action::lend::receive_interest_rate),
                        )
                        .branch(
                            case![ActionState::ReceiveTransferFromAccountTag { strategy, action }]
                                .endpoint(message::strategy::action::transfer::receive_account_tag),
                        )
                        .branch(
                            case![ActionState::ReceiveTransferToAccountTag { strategy, action }]
                                .endpoint(message::strategy::action::transfer::receive_account_tag),
                        )
                        .branch(
                            case![ActionState::ReceivePercentage { strategy, action }]
                                .endpoint(message::strategy::action::receive_percentage),
                        ),
                ),
        )
        .branch(case![State::ReceivePairSymbol].endpoint(message::pair::receive_symbol))
        .branch(case![State::ReceiveLendingCurrency].endpoint(message::lending::receive_currency))
        .branch(dptree::endpoint(message::handler));

    let callback_query_handler = Update::filter_callback_query()
        .filter(user_id_pred)
        .branch(
            case![State::Strategy(state)]
                .branch(
                    case![StrategyState::ReceiveDeleteConfirm { strategy }]
                        .endpoint(callback_query::strategy::receive_delete_confirm),
                )
                .branch(
                    case![StrategyState::Product(state)]
                        .branch(
                            case![ProductState::Receive { strategy }]
                                .endpoint(callback_query::strategy::product::receive),
                        )
                        .branch(
                            case![ProductState::ReceiveBalanceAccountType { strategy, product }]
                                .endpoint(
                                    callback_query::strategy::product::receive_balance_account_type,
                                ),
                        ),
                )
                .branch(
                    case![StrategyState::Condition(state)].branch(
                        case![ConditionState::Receive { strategy }]
                            .endpoint(callback_query::strategy::condition::receive),
                    ),
                )
                .branch(
                    case![StrategyState::Action(state)]
                        .branch(
                            case![ActionState::ReceiveActionNumber { strategy }]
                                .endpoint(callback_query::strategy::action::receive_number),
                        )
                        .branch(
                            case![ActionState::ReceiveActionModif { strategy, index }]
                                .endpoint(callback_query::strategy::action::receive_modif),
                        )
                        .branch(
                            case![ActionState::ReceiveDeleteConfirm { strategy, index }]
                                .endpoint(callback_query::strategy::action::receive_delete_confirm),
                        )
                        .branch(
                            case![ActionState::Receive { strategy }]
                                .endpoint(callback_query::strategy::action::receive),
                        )
                        .branch(
                            case![ActionState::ReceiveOrder { strategy, action }]
                                .endpoint(callback_query::strategy::action::order::receive),
                        )
                        .branch(
                            case![ActionState::ReceiveTransferFrom { strategy, action }]
                                .endpoint(callback_query::strategy::action::transfer::receive),
                        )
                        .branch(
                            case![ActionState::ReceiveTransferTo { strategy, action }]
                                .endpoint(callback_query::strategy::action::transfer::receive),
                        ),
                ),
        )
        .branch(case![State::ReceivePairSymbol].endpoint(callback_query::pair::receive_symbol))
        .branch(
            case![State::ReceiveLendingCurrency]
                .endpoint(callback_query::lending::receive_currency),
        )
        .branch(dptree::endpoint(callback_query::handler));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

fn user_id_pred(upd: Update) -> bool {
    upd.from()
        .is_some_and(|user| user.id.0 == env!("USER_ID").parse::<u64>().unwrap())
}
