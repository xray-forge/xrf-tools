use serde::Serialize;
use xrf_editable_level::EditableSectorItem;

/// One mesh a sector holds: a scene object by name, and one of its object's meshes by name.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneSectorItem {
  pub object: String,
  pub mesh: String,
}

impl From<&EditableSectorItem> for SceneSectorItem {
  fn from(item: &EditableSectorItem) -> Self {
    Self {
      object: item.object.clone(),
      mesh: item.mesh.clone(),
    }
  }
}
