use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt, vec::IntoIter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    Success {
        code: String,
        data: T,

        #[serde(flatten)]
        other: HashMap<String, Value>,
    },
    Error {
        code: String,
        msg: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paginated<T> {
    current_page: u16,
    page_size: u16,
    total_num: u32,
    total_page: u16,
    items: Vec<T>,
}

impl<T> IntoIterator for Paginated<T> {
    type Item = T;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    #[serde(alias = "orderNo")]
    order_id: String,
    client_oid: Option<String>,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let client_oid = match &self.client_oid {
            Some(client_oid) => format!("<b>Client Order ID:</b> {client_oid}"),
            None => Default::default(),
        };

        write!(f, "<b>Order ID:</b> {}\n\n{}", self.order_id, client_oid)
    }
}
