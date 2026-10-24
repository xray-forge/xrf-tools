use xrf_visual::LightsDescription;

use crate::parts::LevelTextureReference;

/// A level's lights, and what each projector a spot names resolved to.
pub struct PackedLevelLights {
  pub lights: LightsDescription,
  pub projectors: Vec<LevelTextureReference>,
}
