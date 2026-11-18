use serde::{Deserialize, Serialize};
use xrf_engine_target::XrayEngineChoice;
use xrf_vfs::XrayRoots;

use crate::scene::SceneSource;

/// A request to open a Level Editor scene: its `.level`, where its library objects are looked for, and the game data it
/// is drawn with.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneOpenRequest {
  pub source: SceneSource,
  /// Directories the scene objects' `.object` files are looked for in, in search order; `objects` beside the directory
  /// holding the `.level` where none are given.
  pub objects_roots: Option<Vec<String>>,
  /// Trees the scene's textures and shaders are searched in; where none are given, `gamedata` beside the `rawdata`
  /// holding the scene's `levels` where it exists, else no game data.
  pub game_roots: Option<XrayRoots>,
  /// Whether the game's configs resolve with the Monolith/Anomaly DLTX patch dialect.
  pub is_dltx: bool,
  /// Which engine the game's configs are read as; detected on `auto`.
  pub engine: XrayEngineChoice,
}
