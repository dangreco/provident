// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use serde::Serialize;

/// Body sent by AccountInfo.aspx: `{"password"}`.
#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct GetKeyRequest {
    pub password: String,
}

/// Redacts `password`.
impl fmt::Debug for GetKeyRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GetKeyRequest")
            .field("password", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts_password() {
        let req = GetKeyRequest {
            password: "hunter2".into(),
        };
        assert!(!format!("{req:?}").contains("hunter2"));
    }
}
