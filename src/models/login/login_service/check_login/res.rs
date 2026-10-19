// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

/// `d` is a bare boolean: `true` when already signed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CheckLoginResponse {
    /// `true` when the session cookie is already authenticated.
    pub logged_in: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_observed_payload() {
        // Observed `d` for a client without a session.
        let res: CheckLoginResponse = serde_json::from_str("false").unwrap();
        assert!(!res.logged_in);
    }
}
