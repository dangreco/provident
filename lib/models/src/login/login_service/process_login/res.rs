// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

use crate::aspnet::JsonString;

/// `d` is the result serialized into a JSON string: `"{ \"success\": false, \"msg\": \"…\" }"`.
pub type ProcessLoginResponse = JsonString<ProcessLoginResult>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessLoginResult {
    pub success: bool,
    /// User-facing reason on failure (e.g. `"Invalid Username or Password"`). Only observed on
    /// failure, so treated as optional.
    #[serde(default)]
    pub msg: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_observed_failure_payload() {
        // Observed `d` for unknown credentials (note the inner JSON is a string).
        let res: ProcessLoginResponse = serde_json::from_str(
            r#""{ \"success\": false, \"msg\": \"Invalid Username or Password\"  }""#,
        )
        .unwrap();
        assert_eq!(
            res.into_inner(),
            ProcessLoginResult {
                success: false,
                msg: Some("Invalid Username or Password".into()),
            }
        );
    }

    #[test]
    fn msg_is_optional() {
        let res: ProcessLoginResponse = serde_json::from_str(r#""{\"success\": true}""#).unwrap();
        assert!(res.success && res.msg.is_none());
    }
}
