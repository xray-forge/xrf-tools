use serde::Serialize;
use xrf_engine_target::XrayEngineChoice;
use xrf_vfs::XrayRoots;

/// The game data a scene is drawn with, which its surfaces' textures and shaders are found in: the trees searched, and
/// how the game's configs in them read, as a compiled level is opened with them. A scene with no game data opens and
/// draws, its textures missing.
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
