use serde::Serialize;

use crate::scene::{SceneObjectId, SceneSpawnModel};

/// A spawn point drawn as its entity's model, as the page is told of it: which object, and the name the renderer knows
/// the model by, which a model the renderer could not read is reported under.
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
