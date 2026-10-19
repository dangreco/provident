// SPDX-License-Identifier: GPL-3.0-only

use std::ops::{Deref, DerefMut};

use serde::de::{DeserializeOwned, Error as _};
use serde::ser::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// JSON value delivered as a JSON *string* (`"{ \"success\": true }"`), as page methods that
/// return a pre-serialized `string` do. Decodes the string, then `T` from its contents.
///
/// ```
/// use provident::models::aspnet::JsonString;
///
/// let v: JsonString<Vec<u8>> = serde_json::from_str(r#""[1, 2]""#).unwrap();
/// assert_eq!(*v, [1, 2]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct JsonString<T>(pub T);

impl<T> JsonString<T> {
    /// Returns the decoded value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for JsonString<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for JsonString<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for JsonString<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        serde_json::from_str(&raw)
            .map(Self)
            .map_err(D::Error::custom)
    }
}

impl<T: Serialize> Serialize for JsonString<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let raw = serde_json::to_string(&self.0).map_err(S::Error::custom)?;
        serializer.serialize_str(&raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unquoted_json() {
        // The value must be a string; a bare array is not this wire format.
        assert!(serde_json::from_str::<JsonString<Vec<u8>>>("[1, 2]").is_err());
    }

    #[test]
    fn rejects_invalid_inner_json() {
        assert!(serde_json::from_str::<JsonString<Vec<u8>>>(r#""[1,""#).is_err());
    }

    #[test]
    fn round_trips() {
        let json = serde_json::to_string(&JsonString(vec![1u8, 2])).unwrap();
        assert_eq!(json, r#""[1,2]""#);
        assert_eq!(
            serde_json::from_str::<JsonString<Vec<u8>>>(&json).unwrap(),
            JsonString(vec![1, 2])
        );
    }
}
