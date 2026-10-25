use serde::Serialize;

use crate::section::key::environment_value::EnvironmentValue;

/// What the engine holds for a key a section does not write: the default it reads with, or for a key it requires, the
/// value its descriptor is constructed with, which is what a viewer shows in place of a section the engine would
/// refuse.
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

/// The default as a value.
impl From<EnvironmentDefault> for EnvironmentValue {
  fn from(default: EnvironmentDefault) -> Self {
    match default {
      EnvironmentDefault::Number(value) => Self::Number(value),
      EnvironmentDefault::Integer(value) => Self::Integer(value),
      EnvironmentDefault::Vector(components) => Self::Vector(components.to_vec()),
      EnvironmentDefault::Flag(value) => Self::Flag(value),
      EnvironmentDefault::Text(value) => Self::Text(value.to_owned()),
      EnvironmentDefault::List => Self::List(Vec::new()),
    }
  }
}
