use serde::Serialize;

use crate::scene::{SceneBinaryToolDescription, SceneProblem};

/// What a scene's binary tools hold, counted, and why any of them did not read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneBinaryToolsDescription {
  /// The binary tools the scene holds a part of that read.
  pub tools: Vec<SceneBinaryToolDescription>,
  /// The binary tools whose parts did not read, each left out of [`Self::tools`].
  pub problems: Vec<SceneProblem>,
}
