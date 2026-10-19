// SPDX-License-Identifier: GPL-3.0-only

use provident_http::Request;
use provident_models::secure::account_info::get_details::{
    self, AccountDetails, GetDetailsResponse,
};

use super::Account;
use crate::Result;

impl Account {
    /// The signed-in user's profile (`GetDetails`).
    #[maybe_async::maybe_async]
    pub async fn details(&self) -> Result<AccountDetails> {
        let request = Request::page_method(get_details::METHOD, get_details::PATH).build();
        let response = self.http.send::<GetDetailsResponse>(request).await?;
        Ok(response.into_inner())
    }
}
