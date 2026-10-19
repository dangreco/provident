// SPDX-License-Identifier: GPL-3.0-only

use provident_http::Request;
use provident_models::api::internal::graphs::quick_graphs::{
    self, QuickGraphsRequest, QuickGraphsResponse,
};

use super::Usage;
use crate::Result;

impl Usage {
    /// Consumption time series (`quickgraphs`): one series per utility, hourly for ranges of up
    /// to 7 days and daily beyond. Empty when no requested meter matched.
    #[maybe_async::maybe_async]
    pub async fn series(&self, query: &QuickGraphsRequest) -> Result<QuickGraphsResponse> {
        let request = Request::get(quick_graphs::PATH).query(query)?.build();
        Ok(self.http.send::<QuickGraphsResponse>(request).await?)
    }
}
