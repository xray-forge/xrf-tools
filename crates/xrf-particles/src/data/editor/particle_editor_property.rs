use serde::{Deserialize, Serialize};

use crate::data::editor::particle_editor_value::ParticleEditorValue;

/// One named property of an editor action, such as a source's `Rate` or a bounce's `Friction`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEditorProperty {
  /// The name the SDK shows and keys the property by, which also orders it when stored.
  pub name: String,
  pub value: ParticleEditorValue,
}

impl ParticleEditorProperty {
  pub fn new(name: &str, value: ParticleEditorValue) -> Self {
    Self {
      name: String::from(name),
      value,
    }
  }
}
