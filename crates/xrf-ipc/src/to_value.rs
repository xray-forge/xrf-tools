use serde::Serialize;

/// A constant's value as it serializes, which is what the frontend's bindings declare it as.
///
/// # Panics
///
/// Panics when the value does not serialize to JSON, as a map with non-string keys does.
pub fn to_value<T: Serialize>(value: &T) -> serde_json::Value {
  serde_json::to_value(value).unwrap_or_else(|error| panic!("An IPC constant does not serialize: {error}"))
}
