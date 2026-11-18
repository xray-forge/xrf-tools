use serde::Serialize;
use xrf_engine_target::XrayEngineChoice;
use xrf_vfs::XrayRoots;

/// The game data a scene's textures and shaders are found in, given as a compiled level's is. A scene with no game data
/// still opens and draws, its textures missing.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneGameData {
  /// Trees the scene's textures and shaders are searched in, in search order; none for a scene with no game data.
  pub roots: XrayRoots,
  /// Whether the game's configs resolve with the Monolith/Anomaly DLTX patch dialect.
  pub is_dltx: bool,
  /// Which engine the game's configs are read as; detected on `auto`.
  pub engine: XrayEngineChoice,
}
