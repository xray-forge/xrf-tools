use std::time::Instant;

use glam::Vec3;
use xrf_renderer::RenderSceneId;

use crate::contract::sound::world_sound_toggles::WorldSoundToggles;
use crate::sound::rain_sound::RainFrame;
use crate::sound::thunder_sound::ThunderSound;
use crate::weather::weather_ambients::WeatherAmbients;

/// What the viewport a scene is heard by gives its sound for a frame: the listening viewport where it shows the scene,
/// else the one driving it.
#[derive(Clone, Copy)]
pub struct SoundDrive<'a> {
  pub scene: RenderSceneId,
  /// The frame's moment, which strikes are stamped by.
  pub at: Instant,
  /// Whether the scene is heard.
  pub is_audible: bool,
  /// The camera, in engine space: where it stands, where it looks, which way is up.
  pub eye: Vec3,
  pub forward: Vec3,
  pub up: Vec3,
  /// Milliseconds of the day the viewport's weather stands at.
  pub game_time: u32,
  pub toggles: WorldSoundToggles,
  /// The ambients its weather plays between, none while nothing plays.
  pub ambients: Option<&'a WeatherAmbients>,
  pub rain: RainFrame,
  /// The strikes the viewport's weather thundered with since its last frame.
  pub thunder: &'a [ThunderSound],
}
