// SPDX-License-Identifier: GPL-3.0-only

//! `DateTime` encoded as `"\/Date(<ms since epoch>[+-HHMM])\/"`.
//!
//! The milliseconds are always UTC; the optional offset suffix is informational and ignored.
//! Serializes without an offset.
//!
//! ```
//! use chrono::{DateTime, Utc};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Reading {
//!     #[serde(with = "provident::models::aspnet::date")]
//!     at: DateTime<Utc>,
//! }
//!
//! let r: Reading = serde_json::from_str(r#"{"at":"\/Date(1700000000000)\/"}"#).unwrap();
//! assert_eq!(r.at.timestamp(), 1_700_000_000);
//! ```

use chrono::{DateTime, Utc};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serializer};

const PREFIX: &str = "/Date(";
const SUFFIX: &str = ")/";

/// Serializes as `/Date(<ms since epoch>)/`, without an offset.
pub fn serialize<S: Serializer>(value: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&format_args!(
        "{PREFIX}{}{SUFFIX}",
        value.timestamp_millis()
    ))
}

/// Deserializes a `/Date(<ms>[+-HHMM])/` string into a UTC `DateTime`, ignoring any offset suffix.
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<Utc>, D::Error> {
    let raw = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
    parse(&raw).ok_or_else(|| D::Error::custom(format_args!("invalid ASP.NET date: {raw:?}")))
}

fn parse(raw: &str) -> Option<DateTime<Utc>> {
    let inner = raw.strip_prefix(PREFIX)?.strip_suffix(SUFFIX)?;
    // Skip a leading '-' (pre-1970) before looking for the offset sign.
    let end = inner
        .get(1..)?
        .find(['+', '-'])
        .map_or(inner.len(), |i| i + 1);
    let (millis, offset) = inner.split_at(end);
    if !offset.is_empty() && !(offset.len() == 5 && offset[1..].bytes().all(|b| b.is_ascii_digit()))
    {
        return None;
    }
    DateTime::from_timestamp_millis(millis.parse().ok()?)
}

/// [`date`](self) for `Option<DateTime<Utc>>`; JSON `null` ↔ `None`.
pub mod option {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serializes `Some` as a `/Date(…)/` string and `None` as JSON `null`.
    pub fn serialize<S: Serializer>(
        value: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => super::serialize(value, serializer),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes an optional `/Date(…)/` string; JSON `null` becomes `None`.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        #[derive(Deserialize)]
        struct Wrapper(#[serde(with = "super")] DateTime<Utc>);
        Ok(Option::<Wrapper>::deserialize(deserializer)?.map(|Wrapper(value)| value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct T {
        #[serde(with = "super")]
        at: DateTime<Utc>,
        #[serde(with = "super::option")]
        maybe: Option<DateTime<Utc>>,
    }

    fn at(ms: i64) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(ms).unwrap()
    }

    #[test]
    fn parses_escaped_slashes_offset_and_negative() {
        let t: T = serde_json::from_str(
            r#"{"at":"\/Date(1700000000123+0100)\/","maybe":"/Date(-86400000)/"}"#,
        )
        .unwrap();
        // Offset does not shift the instant.
        assert_eq!(t.at, at(1_700_000_000_123));
        assert_eq!(t.maybe, Some(at(-86_400_000)));
        assert_eq!(parse("/Date(-86400000-0500)/"), Some(at(-86_400_000)));
    }

    #[test]
    fn round_trips() {
        let t = T {
            at: at(1_700_000_000_123),
            maybe: None,
        };
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"at":"/Date(1700000000123)/","maybe":null}"#);
        assert_eq!(serde_json::from_str::<T>(&json).unwrap(), t);
    }

    #[test]
    fn rejects_malformed() {
        for raw in [
            "2023-11-14",
            "/Date()/",
            "/Date(12x)/",
            "/Date(1+01)/",
            "/Date(1)",
        ] {
            assert_eq!(parse(raw), None, "{raw}");
        }
    }
}
