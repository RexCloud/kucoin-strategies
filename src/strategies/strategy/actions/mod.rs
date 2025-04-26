use std::ops::Deref;

use crate::kucoin::{
    account::Transfer,
    lending::{Lend, Redeem},
    request::Request,
    trading::spot::order,
    KuCoin,
};

mod action;
pub use action::{Action, ActionKind};

#[derive(Debug, Default, Clone)]
pub struct Actions(Vec<Action>);

impl Actions {
    pub fn add(&mut self, action: Action) {
        self.0.push(action);
    }

    pub fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }

    pub fn r#move(&mut self, index: usize, up: bool) {
        let b = index.saturating_add_signed(if up { -1 } else { 1 });

        if self.get(index).is_some() && self.get(b).is_some() {
            self.0.swap(index, b);
        }
    }

    pub fn executable(&mut self, kucoin: &KuCoin) -> Option<(&mut Action, Request)> {
        self.0.iter_mut().find_map(|action| {
            Request::try_from((action.deref(), kucoin))
                .map(|request| (action, request))
                .ok()
        })
    }
}

impl Deref for Actions {
    type Target = Vec<Action>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<(&Action, &KuCoin)> for Request {
    type Error = ();

    fn try_from((action, kucoin): (&Action, &KuCoin)) -> std::result::Result<Self, Self::Error> {
        if action.skip() {
            return Err(());
        }

        match action.amount(kucoin) {
            Some(amount) => {
                let symbol = action.symbol().to_string();

                match action.kind() {
                    ActionKind::SpotOrder { side, price, .. } => Ok(match price {
                        Some(price) => order::Add::limit(symbol, *side, *price, amount),
                        None => order::Add::market(symbol, *side, amount),
                    }
                    .into()),
                    ActionKind::Lend { interest_rate } => {
                        Ok(Lend::new(symbol, *interest_rate, amount).into())
                    }
                    ActionKind::Redeem => match kucoin.lending().orders().get(&symbol) {
                        Some(order) => {
                            Ok(
                                Redeem::new(symbol, order.purchase_order_no().to_string(), amount)
                                    .into(),
                            )
                        }
                        None => Err(()),
                    },
                    ActionKind::Transfer {
                        from,
                        to,
                        from_account_tag,
                        to_account_tag,
                    } => Ok(Transfer::internal(
                        symbol,
                        amount,
                        *from,
                        *to,
                        from_account_tag.clone(),
                        to_account_tag.clone(),
                    )
                    .into()),
                }
            }
            None => Err(()),
        }
    }
}
