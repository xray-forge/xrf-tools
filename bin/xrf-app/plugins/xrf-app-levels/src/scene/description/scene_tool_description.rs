use serde::Serialize;

use crate::scene::{ScenePartFileDescription, SceneToolFormat, SceneToolKind};

/// One tool a scene holds part files of, as its open listed them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneToolDescription {
  /// The tool's name, which names its files: `scene_object`, `spawn`, `ai_map` and the rest.
  pub name: String,
  /// The editor's tool of that name; none for a tool the editor does not save.
  pub kind: Option<SceneToolKind>,
  /// How the tool saves its part; none for a tool the editor does not know whose files do not read.
  pub format: Option<SceneToolFormat>,
  /// Its files' sizes summed, in bytes.
  pub size: u64,
  /// Its files in the editor's read order, including those it skips.
  pub files: Vec<ScenePartFileDescription>,
}
