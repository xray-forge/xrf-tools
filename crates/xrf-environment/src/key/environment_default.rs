use serde::Serialize;

use crate::key::environment_value::EnvironmentValue;

/// What the engine holds for a key a section does not write: the default it reads with, or for a key it requires, the
/// value its descriptor is constructed with, which is what a viewer shows in place of a section the engine would refuse.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum EnvironmentDefault {
  Number(f32),
  Integer(i32),
  Vector(&'static [f32]),
  Flag(bool),
  Text(&'static str),
  /// An empty list.
  List,
}

impl EnvironmentDefault {
  /// The default as a value.
  pub fn to_value(self) -> EnvironmentValue {
    match self {
      Self::Number(value) => EnvironmentValue::Number(value),
      Self::Integer(value) => EnvironmentValue::Integer(value),
      Self::Vector(components) => EnvironmentValue::Vector(components.to_vec()),
      Self::Flag(value) => EnvironmentValue::Flag(value),
      Self::Text(value) => EnvironmentValue::Text(value.to_owned()),
      Self::List => EnvironmentValue::List(Vec::new()),
    }
  }
}
