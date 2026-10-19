// SPDX-License-Identifier: GPL-3.0-only

use crate::http::Request;
use crate::models::secure::account_info::get_key::{self, GetKeyRequest, GetKeyResponse};

use super::Account;
use crate::{Error, Result};

impl Account {
    /// The account's API key (`GetKey`). The portal requires re-entering the account password.
    ///
    /// # Errors
    /// [`Error::PasswordRejected`] when the server refuses the password (it still answers 200).
    #[maybe_async::maybe_async]
    pub async fn api_key(&self, password: &str) -> Result<String> {
        let body = GetKeyRequest {
            password: password.to_owned(),
        };
        let request = Request::page_method(get_key::METHOD, get_key::PATH)
            .json(&body)?
            .build();
        let result = self
            .http
            .send::<GetKeyResponse>(request)
            .await?
            .into_inner();
        match result.api_key {
            Some(key) if !result.error => Ok(key),
            _ => Err(Error::PasswordRejected {
                message: result.msg,
            }),
        }
    }
}
