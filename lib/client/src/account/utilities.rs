// SPDX-License-Identifier: GPL-3.0-only

use provident_http::Request;
use provident_models::common::Utility;
use provident_models::secure::dashboard::default::get_utilities::{
    self, GetUtilitiesRequest, GetUtilitiesResponse,
};

use super::Account;
use crate::Result;

impl Account {
    /// Utilities metered for this account (`GetUtilities`), in the order the portal lists them.
    #[maybe_async::maybe_async]
    pub async fn utilities(&self) -> Result<Vec<Utility>> {
        let request = Request::page_method(get_utilities::METHOD, get_utilities::PATH)
            .json(&GetUtilitiesRequest {})?
            .build();
        let response = self.http.send::<GetUtilitiesResponse>(request).await?;
        Ok(response.into_inner())
    }
}
