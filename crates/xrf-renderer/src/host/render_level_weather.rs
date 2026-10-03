use std::collections::HashMap;

use xrf_engine_target::XrayEngine;
use xrf_environment::{SunPosition, WeatherDescriptor, WeatherModifier};

use crate::host::render_lens_flare::RenderLensFlare;
use crate::host::render_rain::RenderRain;
use crate::host::render_thunder::RenderThunder;
use crate::host::render_wet_surfaces::RenderWetSurfaces;

/// What any weather of an open level plays with, whatever cycle it plays.
#[derive(Clone, Debug, Default)]
pub struct RenderLevelWeather {
  /// The engine its configs are read as, whose sky and shading the level is drawn with.
  pub engine: XrayEngine,
  /// Every weather effect of its game, by name, each sorted by its time from the effect's start.
  pub effects: HashMap<String, Vec<WeatherDescriptor>>,
  /// The level's `level.env_mod` volumes.
  pub modifiers: Vec<WeatherModifier>,
  /// Monolith's table of where the sun stands, midnight first; none on OpenXRay.
  pub sun_table: Option<Vec<SunPosition>>,
  /// What its rain is drawn with, none for a level drawing no rain.
  pub rain: Option<RenderRain>,
  /// What its rain wets surfaces with.
  pub wet: Option<RenderWetSurfaces>,
  /// What its weather strikes with.
  pub thunder: Option<RenderThunder>,
  /// Every lens flare a keyframe may name by `sun`, by its section's name.
  pub suns: HashMap<String, RenderLensFlare>,
}
