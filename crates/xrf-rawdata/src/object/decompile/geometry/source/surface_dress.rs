use serde::{Deserialize, Serialize};

/// A surface's compiler shader (`shaders_xrlc.xr`) and game material (`gamemtl.xr`). No OGF keeps either, so a
/// decompiled surface takes the editor's defaults unless a level's collision form and lighting say more.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceDress {
  pub compiler_shader: String,
  pub game_material: String,
}

impl SurfaceDress {
  /// The Actor Editor's compiler shader and game material for a new surface.
  pub const DEFAULT_SETTING: &'static str = "default";
}

impl Default for SurfaceDress {
  fn default() -> Self {
    Self {
      compiler_shader: String::from(Self::DEFAULT_SETTING),
      game_material: String::from(Self::DEFAULT_SETTING),
    }
  }
}
