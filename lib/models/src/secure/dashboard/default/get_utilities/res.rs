// SPDX-License-Identifier: GPL-3.0-only

use crate::aspnet::JsonString;
use crate::common::Utility;

/// `d` is the list serialized into a JSON string: `"[\"Cold Water\",\"Electricity\"]"`.
pub type GetUtilitiesResponse = JsonString<Vec<Utility>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_observed_payload() {
        // Observed `d` for an account with these three utilities.
        let res: GetUtilitiesResponse =
            serde_json::from_str(r#""[\"Cold Water\",\"Electricity\",\"Hot Water\"]""#).unwrap();
        assert_eq!(
            res.into_inner(),
            [Utility::ColdWater, Utility::Electricity, Utility::HotWater]
        );
    }

    #[test]
    fn unknown_utility_fails_decoding() {
        let res = serde_json::from_str::<GetUtilitiesResponse>(r#""[\"Steam\"]""#);
        assert!(res.is_err());
    }
}
