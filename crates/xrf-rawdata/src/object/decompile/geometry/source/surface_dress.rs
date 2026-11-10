use serde::{Deserialize, Serialize};

/// What a surface is built and played with besides its texture and engine shader: the compiler shader the level
/// compiler lights and collides it by (`shaders_xrlc.xr`), and the game material bullets, steps and physics read
/// (`gamemtl.xr`). No OGF keeps either, so a decompiled surface takes the editor's defaults unless a level's collision
/// form and lighting say more.
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
