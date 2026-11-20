use serde::Serialize;

use crate::level::build::level_exported_previous::LevelExportedPrevious;

/// One level file an export wrote.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelExportedFile {
  pub name: String,
  pub size: usize,
  pub previous: LevelExportedPrevious,
}
