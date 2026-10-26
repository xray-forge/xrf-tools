use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use xrf_error::XrfError;

/// A float kept as the bits it is stored in, for state the SDK writes without initialising: a vanilla spawn's
/// physics states hold NaNs with payloads, and whole states of `0xffffffff`, which no decimal brings back.
///
/// A finite value travels as its number, in LTX and JSON alike, and anything else as its bits in hex (`0xffffffff`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[cfg_attr(feature = "typescript-bindings", specta(type = RawFloatValue))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawFloat(pub u32);

// What a raw float serialises as, declared for the TypeScript mirror alone and inlined into it. The number is declared
// as an integer because Specta writes a float as `number | null`, and a raw float is never null.
#[cfg(feature = "typescript-bindings")]
#[derive(specta::Type, Serialize)]
#[specta(inline)]
#[serde(untagged)]
#[allow(dead_code)]
enum RawFloatValue {
  Value(i32),
  Bits(String),
}

impl RawFloat {
  pub fn get(self) -> f32 {
    f32::from_bits(self.0)
  }

  /// The value's bits as hex, which is how one with no finite value is written.
  fn to_hex(self) -> String {
    format!("0x{:08x}", self.0)
  }
}

impl From<f32> for RawFloat {
  fn from(value: f32) -> Self {
    Self(value.to_bits())
  }
}

impl Display for RawFloat {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    let value: f32 = self.get();

    if value.is_finite() {
      write!(formatter, "{value}")
    } else {
      formatter.write_str(&self.to_hex())
    }
  }
}

impl FromStr for RawFloat {
  type Err = XrfError;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    let string: &str = string.trim();

    if let Some(hex) = string.strip_prefix("0x") {
      return u32::from_str_radix(hex, 16)
        .map(Self)
        .map_err(|error| XrfError::new_parsing_error(format!("Failed to parse float bits '{string}': {error}")));
    }

    string
      .parse::<f32>()
      .map(Self::from)
      .map_err(|error| XrfError::new_parsing_error(format!("Failed to parse float '{string}': {error}")))
  }
}

impl Serialize for RawFloat {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let value: f32 = self.get();

    if value.is_finite() {
      serializer.serialize_f32(value)
    } else {
      serializer.serialize_str(&self.to_hex())
    }
  }
}

impl<'de> Deserialize<'de> for RawFloat {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    deserializer.deserialize_any(RawFloatVisitor)
  }
}

/// Takes a [`RawFloat`] from a number or from the string of its bits.
struct RawFloatVisitor;

impl Visitor<'_> for RawFloatVisitor {
  type Value = RawFloat;

  fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    formatter.write_str("a number, or a float's bits as a hex string")
  }

  fn visit_f64<E: de::Error>(self, value: f64) -> Result<RawFloat, E> {
    Ok(RawFloat::from(value as f32))
  }

  fn visit_i64<E: de::Error>(self, value: i64) -> Result<RawFloat, E> {
    Ok(RawFloat::from(value as f32))
  }

  fn visit_u64<E: de::Error>(self, value: u64) -> Result<RawFloat, E> {
    Ok(RawFloat::from(value as f32))
  }

  fn visit_str<E: de::Error>(self, value: &str) -> Result<RawFloat, E> {
    value.parse().map_err(E::custom)
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfResult;

  use crate::data::physics::raw_float::RawFloat;

  #[test]
  fn keeps_the_bits_of_a_nan_through_text_and_json() -> XrfResult {
    for bits in [0xffff_ffff_u32, 0xffcc_031b, 0x7fc0_0000, 0x7f80_0000, 0xff80_0000] {
      let value: RawFloat = RawFloat(bits);

      assert_eq!(value.to_string(), format!("0x{bits:08x}"));
      assert_eq!(value.to_string().parse::<RawFloat>()?, value);
      assert_eq!(
        serde_json::from_str::<RawFloat>(&serde_json::to_string(&value)?)?,
        value
      );
    }

    Ok(())
  }

  #[test]
  fn writes_a_finite_value_as_its_number() -> XrfResult {
    for value in [0.0_f32, -0.0, 1.5, -224.76178, 3.3e-37, 1.778e-42, f32::MAX] {
      let raw: RawFloat = RawFloat::from(value);

      assert_eq!(raw.to_string(), value.to_string());
      assert_eq!(raw.to_string().parse::<RawFloat>()?, raw);
      assert_eq!(serde_json::to_string(&raw)?, serde_json::to_string(&value)?);
      assert_eq!(serde_json::from_str::<RawFloat>(&serde_json::to_string(&raw)?)?, raw);
    }

    Ok(())
  }

  #[test]
  fn reads_a_whole_number_from_json() -> XrfResult {
    assert_eq!(serde_json::from_str::<RawFloat>("3")?, RawFloat::from(3.0));
    assert_eq!(serde_json::from_str::<RawFloat>("-2")?, RawFloat::from(-2.0));

    Ok(())
  }

  #[test]
  fn refuses_text_that_is_neither() {
    assert!("0xnothex".parse::<RawFloat>().is_err());
    assert!("north".parse::<RawFloat>().is_err());
  }
}
