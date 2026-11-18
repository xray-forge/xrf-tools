use serde::Serialize;

use crate::scene::SceneProblemKind;

/// Something of a scene the Level Editor would skip or refuse, found by the open or a part's read and kept on its tool;
/// never a refused open.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneProblem {
  pub kind: SceneProblemKind,
  /// The tool it belongs to, by the name its part files carry: `scene_object`, `spawn`, `ai_map` and the rest.
  pub tool: String,
  /// The part file it was found in, `<tool>.part` or a numbered one; none where the tool's objects as a whole, group
  /// members included, do not read.
  pub file: Option<String>,
  pub message: String,
}
