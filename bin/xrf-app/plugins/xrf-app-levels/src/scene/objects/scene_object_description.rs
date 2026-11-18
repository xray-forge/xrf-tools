use serde::Serialize;
use xrf_editable_level::EditableCustomObject;
use xrf_vfs::XraySearch;

use crate::scene::{SceneObjectId, SceneObjectProperties, SceneObjectTransform, SceneToolObject};

/// One object of a scene's object tool, as the outliner lists it and an inspector shows it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneObjectDescription {
  pub id: SceneObjectId,
  /// Unique within the scene.
  pub name: String,
  /// The name of the group it belongs to; none for an object of its tool's own part.
  pub group: Option<String>,
  /// `co_flags`: motion, grouping and the editor's other states.
  pub flags: u32,
  pub transform: SceneObjectTransform,
  pub properties: SceneObjectProperties,
}

impl SceneObjectDescription {
  /// An ungrouped object `id` from its common fields `custom` and its kind's `properties`.
  pub(crate) fn new(id: SceneObjectId, custom: &EditableCustomObject, properties: SceneObjectProperties) -> Self {
    Self {
      id,
      name: custom.name.clone(),
      group: None,
      flags: custom.flags,
      transform: SceneObjectTransform::from(&custom.transform),
      properties,
    }
  }

  /// An ungrouped typed object `id`, its library object, if it places one, looked for through `objects`.
  pub(crate) fn of(id: SceneObjectId, object: &impl SceneToolObject, objects: &XraySearch) -> Self {
    Self::new(id, object.get_custom(), object.describe(objects))
  }
}
