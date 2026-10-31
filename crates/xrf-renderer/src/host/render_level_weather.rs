use std::collections::HashMap;

use crate::host::render_lens_flare::RenderLensFlare;
use crate::host::render_rain::RenderRain;
use crate::host::render_thunder::RenderThunder;
use crate::host::render_wet_surfaces::RenderWetSurfaces;

/// What the renderer draws any weather of an open level with, whatever cycle it plays.
#[derive(Clone, Debug, Default)]
pub struct RenderLevelWeather {
  /// What its rain is drawn with, none for a level drawing no rain.
  pub rain: Option<RenderRain>,
  /// What its rain wets surfaces with.
  pub wet: Option<RenderWetSurfaces>,
  /// What its weather strikes with.
  pub thunder: Option<RenderThunder>,
  /// Every lens flare a keyframe may name by `sun`, by its section's name.
  pub suns: HashMap<String, RenderLensFlare>,
}
