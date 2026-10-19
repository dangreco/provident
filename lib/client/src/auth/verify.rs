// SPDX-License-Identifier: GPL-3.0-only

use provident_http::Request;
use provident_models::login::login_service::check_login::{self, CheckLoginResponse};

use super::Auth;
use crate::Result;

impl Auth {
    /// Whether the client currently holds a valid session (`CheckLogin`).
    #[maybe_async::maybe_async]
    pub async fn verify(&self) -> Result<bool> {
        let request = Request::page_method(check_login::METHOD, check_login::PATH).build();
        let response = self.http.send::<CheckLoginResponse>(request).await?;
        Ok(response.logged_in)
    }
}
