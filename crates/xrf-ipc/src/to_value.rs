use serde::Serialize;

/// A constant's value as it serializes, which is what the frontend's bindings declare it as; an error for one that
/// does not serialize to JSON, as a map with non-string keys does not.
pub fn to_value<T: Serialize>(value: &T) -> serde_json::Result<serde_json::Value> {
  // Through the text rather than `serde_json::to_value`, which widens an `f32` to `f64` and so gives `0.1` as
  // `0.10000000149011612`: the serializer writes an `f32` in its own shortest form.
  serde_json::from_str(&serde_json::to_string(value)?)
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use crate::to_value;

  #[test]
  fn keeps_an_f32_in_its_shortest_form() {
    assert_eq!(to_value(&0.1_f32).unwrap().to_string(), "0.1");
    assert_eq!(to_value(&u64::MAX).unwrap(), json!(u64::MAX));
    assert_eq!(to_value(&f32::NAN).unwrap(), json!(null));
  }
}
