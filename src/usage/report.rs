// SPDX-License-Identifier: GPL-3.0-only

use crate::http::Request;
use crate::models::api::internal::reports::quick_report::{
    self, QuickReportRequest, QuickReportResponse,
};

use super::Usage;
use crate::Result;

impl Usage {
    /// Start/end readings and consumption totals (`quickreport`), keyed by utility label.
    /// Rows with `error` set have missing or incomplete readings for the range.
    #[maybe_async::maybe_async]
    pub async fn report(&self, query: &QuickReportRequest) -> Result<QuickReportResponse> {
        let request = Request::get(quick_report::PATH).query(query)?.build();
        Ok(self.http.send::<QuickReportResponse>(request).await?)
    }
}
