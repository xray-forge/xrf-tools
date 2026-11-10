use std::path::PathBuf;

use crate::level::level_surface_inference::LevelSurfaces;

/// What a level's decompile takes besides the level's directory (decision 21's one options type; libraries for
/// placed objects and an output root join it as their stages land).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelDecompileOptions {
  /// The level's name: its scene's main file is `<name>.level`, its parts are in `<name>/`, its objects under
  /// `levels\<name>\`, and its level options build into and prefix with it.
  pub name: String,
  /// The front-end decompiling, such as `xrf-cli level decompile`, and its version, which the sidecars record.
  pub tool: String,
  pub tool_version: String,
  /// A root spawn (`all.spawn`) whose objects on this level stand in for `level.spawn`'s entities (decision 10). The
  /// level is found in it by its directory's name.
  pub root_spawn: Option<PathBuf>,
  /// How the static geometry's surfaces are dressed (decision 14). Inferring reads the level's `level.cform` where the
  /// directory holds one, and needs both libraries below.
  pub surfaces: LevelSurfaces,
  /// The game material library (`gamemtl.xr`) naming the collision form's materials.
  pub game_materials: Option<PathBuf>,
  /// The compiler shader library (`shaders_xrlc.xr`) the inferred compiler shaders are chosen from.
  pub compiler_shaders: Option<PathBuf>,
  /// Object library roots (decision 13) whose surfaces' pairings of engine and compiler shaders break ties between
  /// compiler shaders that fit.
  pub libraries: Vec<PathBuf>,
}

impl LevelDecompileOptions {
  /// The options of a decompile of the level `name` by `tool` at `tool_version`, taking everything else by default:
  /// entities from the spawn lists, surfaces inferred, no library named.
  pub fn new(name: impl Into<String>, tool: impl Into<String>, tool_version: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      tool: tool.into(),
      tool_version: tool_version.into(),
      root_spawn: None,
      surfaces: LevelSurfaces::default(),
      game_materials: None,
      compiler_shaders: None,
      libraries: Vec::new(),
    }
  }
}
