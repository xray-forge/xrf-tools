use serde::Serialize;

use crate::ambient::ambient::Ambient;

/// One level's own ambients, `environment\ambients\<level>.ltx`, which `load_level_specific_ambients` reads over the
/// shared ones of the same name while that level is loaded.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAmbients {
  /// The level's name, which the file is named for.
  pub level: String,
  /// The config, as a logical path.
  pub file: String,
  pub ambients: Vec<Ambient>,
}
