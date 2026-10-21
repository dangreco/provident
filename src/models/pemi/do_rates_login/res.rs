// SPDX-License-Identifier: GPL-3.0-only

use serde::Deserialize;

/// Outcome of a rates login attempt; the site answers `{"response": "ok"|"no", ...}`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RatesLoginResponse {
    /// `"ok"` when the location id was accepted.
    pub response: String,
    /// User-facing message; `"Invalid login."` on failure, empty on success.
    #[serde(default)]
    pub message: String,
}

impl RatesLoginResponse {
    /// `true` when the rates site accepted the credentials.
    pub fn accepted(&self) -> bool {
        self.response == "ok"
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_observed_payloads() {
        let ok: RatesLoginResponse =
            serde_json::from_str(r#"{"response":"ok","message":"","error":0}"#).unwrap();
        assert!(ok.accepted());
        let no: RatesLoginResponse =
            serde_json::from_str(r#"{"response":"no","message":"Invalid login.","error":0}"#)
                .unwrap();
        assert!(!no.accepted());
        assert_eq!(no.message, "Invalid login.");
    }
}
