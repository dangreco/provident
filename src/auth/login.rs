// SPDX-License-Identifier: GPL-3.0-only

use crate::http::Request;
use crate::models::login::login_service::process_login::{
    self, ProcessLoginRequest, ProcessLoginResponse,
};

use super::Auth;
use crate::{Error, Result};

impl Auth {
    /// Signs in (`ProcessLogin`). On success the session cookie is stored in the client and
    /// sent with every later request.
    ///
    /// # Errors
    /// [`Error::LoginRejected`] when the server refuses the credentials (it still answers 200).
    #[maybe_async::maybe_async]
    pub async fn login(&self, request: &ProcessLoginRequest) -> Result<()> {
        let request = Request::page_method(process_login::METHOD, process_login::PATH)
            .json(request)?
            .build();
        let result = self
            .http
            .send::<ProcessLoginResponse>(request)
            .await?
            .into_inner();
        if result.success {
            Ok(())
        } else {
            Err(Error::LoginRejected {
                message: result.msg,
            })
        }
    }
}
