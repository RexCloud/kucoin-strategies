use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    currency: String,
    name: String,
    full_name: String,
    precision: u8,
    confirms: Option<u8>,
    contract_address: Option<String>,
    is_margin_enabled: bool,
    is_debit_enabled: bool,
    chains: Option<Vec<Chain>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chain {
    chain_id: String,
    chain_name: String,
    confirms: u16,
    contract_address: String,
    #[serde(default)]
    deposit_fee_rate: Option<String>,
    deposit_min_size: Option<String>,
    #[serde(default)]
    deposit_tier_fee: Option<String>,
    is_deposit_enabled: bool,
    is_withdraw_enabled: bool,
    max_deposit: Option<String>,
    max_withdraw: Option<String>,
    need_tag: bool,
    pre_confirms: u16,
    withdrawal_min_fee: Option<String>,
    withdrawal_min_size: Option<String>,
    #[serde(default)]
    withdraw_fee_rate: Option<String>,
    #[serde(default)]
    withdraw_max_fee: Option<String>,
    withdraw_precision: u8,
}

impl Currency {
    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn precision(&self) -> u8 {
        self.precision
    }
}
