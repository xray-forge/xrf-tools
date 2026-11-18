use serde::Serialize;

use crate::scene::{SceneObjectId, SceneSpawnModel};

/// A spawn point drawn as its entity's model: the object, and the renderer's name for the model, under which a model
/// the renderer could not read is reported.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneSpawnModelDescription {
  pub object: SceneObjectId,
  pub model: String,
}

impl From<&SceneSpawnModel> for SceneSpawnModelDescription {
  fn from(model: &SceneSpawnModel) -> Self {
    Self {
      object: model.id.clone(),
      model: model.get_model_name(),
    }
  }
}
