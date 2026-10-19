// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

use crate::models::aspnet::JsonString;

/// `d` is the profile serialized into a JSON string.
pub type GetDetailsResponse = JsonString<AccountDetails>;

/// Profile of the signed-in account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDetails {
    /// Login name, e.g. `587-3105`.
    pub login: String,
    /// Given name.
    pub first_name: String,
    /// Family name.
    pub last_name: String,
    /// Contact email address, if the account has one.
    pub email: Option<String>,
    /// Names of the groups the account belongs to (its suites/units), e.g. `["3105"]`.
    pub groups: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_observed_shape() {
        let res: GetDetailsResponse = serde_json::from_str(
            r#""{\"login\":\"1-101\",\"firstName\":\"Ada\",\"lastName\":\"Lovelace\",\"email\":null,\"groups\":[\"101\"]}""#,
        )
        .unwrap();
        let details = res.into_inner();
        assert_eq!(details.login, "1-101");
        assert_eq!((details.email, details.groups), (None, vec!["101".into()]));
    }
}
