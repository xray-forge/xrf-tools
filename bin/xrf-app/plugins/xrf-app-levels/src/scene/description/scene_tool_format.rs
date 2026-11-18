use serde::Serialize;
use xrf_editable_level::EditableLevelPart;

/// How a tool saves its part: as text holding one section per object, or as chunks holding the tool's own stream.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SceneToolFormat {
  /// An object tool's text part: scene objects, sectors, spawn points, groups and the rest.
  Object,
  /// A binary part: the AI map, the detail objects and the wall marks.
  Binary,
}

/// The format a part's first byte tells, for a tool the editor does not know.
impl From<&EditableLevelPart> for SceneToolFormat {
  fn from(part: &EditableLevelPart) -> Self {
    match part {
      EditableLevelPart::Text(_) => Self::Object,
      EditableLevelPart::Binary(_) => Self::Binary,
    }
  }
}
