use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use hmac::{Hmac, Mac};
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    Client, Method,
};
use serde::{de::DeserializeOwned, Serialize};
use sha2::Sha256;
use std::{
    borrow::Cow,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::kucoin::{constants::BASE_URL, Response};

pub struct Request {
    method: Method,
    path: Cow<'static, str>,
    json: String,
}

impl Request {
    fn new<S>(method: Method, path: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            method,
            path: path.into(),
            json: Default::default(),
        }
    }

    pub fn get<S>(path: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Request::new(Method::GET, path)
    }

    pub fn post<S>(path: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Request::new(Method::POST, path)
    }

    pub fn delete<S>(path: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Request::new(Method::DELETE, path)
    }

    pub fn json<T>(mut self, value: &T) -> Self
    where
        T: ?Sized + Serialize,
    {
        self.json = serde_json::to_string(value).unwrap_or_default();
        self
    }

    pub async fn send<T: DeserializeOwned>(self, client: &Client) -> Result<T> {
        let url = BASE_URL.to_string() + &self.path;
        let headers = (&self).into();

        let mut builder = client.request(self.method, url).headers(headers);

        if !self.json.is_empty() {
            builder = builder.body(self.json);
        }

        match builder.send().await?.json().await? {
            Response::Success { data, .. } => Ok(data),
            Response::Error { code, msg } => Err(anyhow!("code: {code}, msg: {msg}")),
        }
    }
}

impl From<&Request> for HeaderMap {
    fn from(req: &Request) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        let str_to_sign = timestamp.clone() + req.method.as_str() + &req.path + &req.json;

        let mut signature: Hmac<Sha256> =
            Hmac::new_from_slice(env!("API_SECRET").as_bytes()).unwrap();

        signature.update(str_to_sign.as_bytes());

        let mut passphrase: Hmac<Sha256> =
            Hmac::new_from_slice(env!("API_SECRET").as_bytes()).unwrap();

        passphrase.update(env!("API_PASSPHRASE").as_bytes());

        let mut headers = HeaderMap::new();

        if !req.json.is_empty() {
            headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        }

        headers.insert(
            "KC-API-SIGN",
            STANDARD
                .encode(signature.finalize().into_bytes())
                .parse()
                .unwrap(),
        );
        headers.insert(
            "KC-API-PASSPHRASE",
            STANDARD
                .encode(passphrase.finalize().into_bytes())
                .parse()
                .unwrap(),
        );
        headers.insert("KC-API-TIMESTAMP", timestamp.parse().unwrap());

        headers
    }
}
