use serde::Serialize;

/// One key's value as the engine holds it after reading.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum EnvironmentValue {
  Number(f32),
  Integer(i32),
  Vector(Vec<f32>),
  Flag(bool),
  Text(String),
  List(Vec<String>),
}

impl EnvironmentValue {
  pub fn as_number(&self) -> Option<f32> {
    match self {
      Self::Number(value) => Some(*value),
      _ => None,
    }
  }

  pub fn as_integer(&self) -> Option<i32> {
    match self {
      Self::Integer(value) => Some(*value),
      _ => None,
    }
  }

  pub fn as_vector(&self) -> Option<&[f32]> {
    match self {
      Self::Vector(components) => Some(components),
      _ => None,
    }
  }

  pub fn as_flag(&self) -> Option<bool> {
    match self {
      Self::Flag(value) => Some(*value),
      _ => None,
    }
  }

  pub fn as_text(&self) -> Option<&str> {
    match self {
      Self::Text(value) => Some(value),
      _ => None,
    }
  }

  pub fn as_list(&self) -> Option<&[String]> {
    match self {
      Self::List(items) => Some(items),
      _ => None,
    }
  }
}
