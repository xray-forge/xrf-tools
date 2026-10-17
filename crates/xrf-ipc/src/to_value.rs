use serde::Serialize;

/// A constant's value as it serializes, which is what the frontend's bindings declare it as; an error for one that
/// does not serialize to JSON, as a map with non-string keys does not.
pub fn to_value<T: Serialize>(value: &T) -> serde_json::Result<serde_json::Value> {
  serde_json::to_value(value)
}
