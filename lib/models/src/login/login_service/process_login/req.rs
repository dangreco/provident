// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use serde::Serialize;

/// Body sent by the site's `login.js`: `{"username", "password", "rememberMe"}`.
#[derive(Clone, PartialEq, Eq, Serialize, bon::Builder)]
#[serde(rename_all = "camelCase")]
pub struct ProcessLoginRequest {
    #[builder(into)]
    pub username: String,
    #[builder(into)]
    pub password: String,
    /// Persistent auth cookie ("Remember me" checkbox).
    #[builder(default)]
    pub remember_me: bool,
}

/// Redacts `password`.
impl fmt::Debug for ProcessLoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcessLoginRequest")
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .field("remember_me", &self.remember_me)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_csharp_parameter_names() {
        let req = ProcessLoginRequest::builder()
            .username("u")
            .password("p")
            .build();
        assert_eq!(
            serde_json::to_value(&req).unwrap(),
            serde_json::json!({"username": "u", "password": "p", "rememberMe": false})
        );
    }

    #[test]
    fn debug_redacts_password() {
        let req = ProcessLoginRequest::builder()
            .username("u")
            .password("hunter2")
            .build();
        assert!(!format!("{req:?}").contains("hunter2"));
    }
}
