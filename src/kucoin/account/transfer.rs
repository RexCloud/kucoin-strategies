use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::kucoin::{account::AccountType, constants::TRANSFER, Request};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    amount: String,
    client_oid: String,
    currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_account_tag: Option<String>,
    from_account_type: AccountType,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to_account_tag: Option<String>,
    to_account_type: AccountType,
    #[serde(skip_serializing_if = "Option::is_none")]
    to_user_id: Option<String>,
    r#type: TransferType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum TransferType {
    Internal,
    ParentToSub,
    SubToParent,
}

impl Transfer {
    pub fn internal(
        currency: String,
        amount: f64,
        from: AccountType,
        to: AccountType,
        from_account_tag: Option<String>,
        to_account_tag: Option<String>,
    ) -> Self {
        Self {
            amount: amount.to_string(),
            client_oid: Uuid::new_v4().simple().to_string(),
            currency,
            from_account_tag,
            from_account_type: from,
            from_user_id: None,
            to_account_tag,
            to_account_type: to,
            to_user_id: None,
            r#type: TransferType::Internal,
        }
    }
}

impl From<Transfer> for Request {
    fn from(value: Transfer) -> Self {
        Request::post(TRANSFER).json(&value)
    }
}
