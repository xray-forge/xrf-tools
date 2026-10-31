use std::collections::HashMap;
use std::sync::Arc;

use xrf_engine_target::XrayEngine;
use xrf_environment::{SunPosition, WeatherDescriptor, WeatherModifier};
use xrf_renderer::RenderLevelWeather;

use crate::host::world_ambient::WorldAmbient;
use crate::host::world_ambient_effect::WorldAmbientEffect;

/// What any weather of an open level plays with, whatever cycle it plays: what the world mixes and schedules, and what
/// the renderer draws it with.
#[derive(Clone, Debug, Default)]
pub struct WorldLevelWeather {
  /// What the renderer draws it with, handed to every frame.
  pub render: Arc<RenderLevelWeather>,
  /// The engine its configs are read as, whose sky and shading the level is drawn with.
  pub engine: XrayEngine,
  /// Every weather effect of its game, by name, each sorted by its time from the effect's start.
  pub effects: HashMap<String, Vec<WeatherDescriptor>>,
  /// The level's `level.env_mod` volumes.
  pub modifiers: Vec<WeatherModifier>,
  /// Monolith's table of where the sun stands, midnight first; none on OpenXRay.
  pub sun_table: Option<Vec<SunPosition>>,
  /// Every ambient a keyframe may name by `ambient`, by its section's name, as it plays on this level.
  pub ambients: HashMap<String, WorldAmbient>,
  /// Every effect an ambient may play, by its section's name.
  pub ambient_effects: HashMap<String, WorldAmbientEffect>,
}
