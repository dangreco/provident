// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A metered utility, as named by the portal (`utilityOptions` in the dashboard script).
///
/// The portal has several names for the same utility (`"Cold Water"` / `"DCW"` / `"Water"`,
/// `"Hot Water"` / `"DHW"`); each variant accepts all of them and serializes to the first.
/// The serialized name is also what endpoints expect when a utility is sent back.
///
/// ```
/// use provident::models::common::Utility;
///
/// let utilities: Vec<Utility> = serde_json::from_str(r#"["DCW", "Electricity", "EV"]"#).unwrap();
/// assert_eq!(utilities, [Utility::ColdWater, Utility::Electricity, Utility::ElectricVehicle]);
/// assert_eq!(serde_json::to_string(&Utility::ColdWater).unwrap(), r#""Cold Water""#);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Utility {
    /// Electrical energy.
    #[serde(rename = "Electricity")]
    Electricity,
    /// Domestic cold water.
    #[serde(rename = "Cold Water", alias = "DCW", alias = "Water")]
    ColdWater,
    /// Domestic hot water.
    #[serde(rename = "Hot Water", alias = "DHW")]
    HotWater,
    /// District or building heating.
    #[serde(rename = "Heating")]
    Heating,
    /// District or building cooling.
    #[serde(rename = "Cooling")]
    Cooling,
    /// Natural gas.
    #[serde(rename = "Gas")]
    Gas,
    /// Electric vehicle charging.
    #[serde(rename = "EV")]
    ElectricVehicle,
}

impl Utility {
    /// Every variant, in declaration order.
    pub const ALL: [Self; 7] = [
        Self::Electricity,
        Self::ColdWater,
        Self::HotWater,
        Self::Heating,
        Self::Cooling,
        Self::Gas,
        Self::ElectricVehicle,
    ];

    /// Name the portal uses for this utility (the serialized form).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Electricity => "Electricity",
            Self::ColdWater => "Cold Water",
            Self::HotWater => "Hot Water",
            Self::Heating => "Heating",
            Self::Cooling => "Cooling",
            Self::Gas => "Gas",
            Self::ElectricVehicle => "EV",
        }
    }
}

impl fmt::Display for Utility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Unknown utility name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown utility {0:?}")]
pub struct UnknownUtility(pub String);

/// Parses any portal name (including aliases such as `DCW`), matching the serde mapping.
impl FromStr for Utility {
    type Err = UnknownUtility;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_owned()))
            .map_err(|_| UnknownUtility(s.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_portal_name_deserializes() {
        // Keys of `utilityOptions` in the dashboard script.
        let cases = [
            ("Electricity", Utility::Electricity),
            ("Cold Water", Utility::ColdWater),
            ("DCW", Utility::ColdWater),
            ("Water", Utility::ColdWater),
            ("Hot Water", Utility::HotWater),
            ("DHW", Utility::HotWater),
            ("Heating", Utility::Heating),
            ("Cooling", Utility::Cooling),
            ("Gas", Utility::Gas),
            ("EV", Utility::ElectricVehicle),
        ];
        for (name, expected) in cases {
            let json = serde_json::to_string(name).unwrap();
            assert_eq!(
                serde_json::from_str::<Utility>(&json).unwrap(),
                expected,
                "{name}"
            );
            assert_eq!(name.parse::<Utility>().unwrap(), expected, "{name}");
        }
    }

    #[test]
    fn serializes_to_canonical_name_and_round_trips() {
        for utility in Utility::ALL {
            let json = serde_json::to_string(&utility).unwrap();
            assert_eq!(json, format!("{:?}", utility.as_str()));
            assert_eq!(serde_json::from_str::<Utility>(&json).unwrap(), utility);
            assert_eq!(utility.to_string().parse::<Utility>().unwrap(), utility);
        }
    }

    #[test]
    fn rejects_unknown_and_rust_names() {
        // The Rust variant name is not a wire name.
        for name in ["ColdWater", "ElectricVehicle", "cold water", "Steam", ""] {
            assert_eq!(
                name.parse::<Utility>(),
                Err(UnknownUtility(name.into())),
                "{name}"
            );
        }
    }
}
