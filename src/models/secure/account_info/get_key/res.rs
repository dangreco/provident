// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::models::aspnet::JsonString;

/// `d` is the result serialized into a JSON string.
pub type GetKeyResponse = JsonString<GetKeyResult>;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetKeyResult {
    /// `true` when the password was rejected.
    pub error: bool,
    /// Reason on error, e.g. `"Invalid password specified."`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    /// Present on success.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// Redacts `api_key`.
impl fmt::Debug for GetKeyResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GetKeyResult")
            .field("error", &self.error)
            .field("msg", &self.msg)
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_both_outcomes() {
        let ok: GetKeyResponse =
            serde_json::from_str(r#""{\"error\":false,\"apiKey\":\"k-123\"}""#).unwrap();
        assert_eq!(ok.api_key.as_deref(), Some("k-123"));
        assert!(!format!("{:?}", *ok).contains("k-123"));

        let bad: GetKeyResponse =
            serde_json::from_str(r#""{\"error\":true,\"msg\":\"Invalid password specified.\"}""#)
                .unwrap();
        assert!(bad.error && bad.api_key.is_none());
        assert_eq!(bad.msg.as_deref(), Some("Invalid password specified."));
    }
}
