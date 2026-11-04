use std::time::Duration;

use xrf_engine_target::XrayEngine;
use xrf_renderer::RenderSceneId;

use crate::contract::sound::world_sound_toggles::WorldSoundToggles;
use crate::host::sound::world_sound_emitter::WorldSoundEmitter;
use crate::host::sound::world_sound_listener::WorldSoundListener;

/// A scene's sound as of a moment on its level's clock: what plays, from where it is heard, and whether it is heard.
/// Frames come from a pool the sink hands back, so a warm world allocates none.
#[derive(Clone, Debug)]
pub struct WorldSoundFrame {
  pub scene: RenderSceneId,
  /// The level's clock as of the frame.
  pub time: Duration,
  /// The engine whose sound render plays it.
  pub engine: XrayEngine,
  /// Whether the listener's viewport shows the scene; every other scene is silent and plays on.
  pub is_audible: bool,
  /// Whether a burst draws the frame at a time of its own, heard by the burst's viewport: the scene's sound plays the
  /// burst's frames from a start of their own, the same however often a burst from that time is drawn.
  pub is_burst: bool,
  pub listener: WorldSoundListener,
  pub toggles: WorldSoundToggles,
  /// What plays at the frame's time, and what started since the frame before, however short.
  pub emitters: Vec<WorldSoundEmitter>,
}

impl Default for WorldSoundFrame {
  fn default() -> Self {
    Self {
      scene: RenderSceneId(0),
      time: Duration::ZERO,
      engine: XrayEngine::default(),
      is_audible: false,
      is_burst: false,
      listener: WorldSoundListener::default(),
      toggles: WorldSoundToggles::default(),
      emitters: Vec::new(),
    }
  }
}

impl WorldSoundFrame {
  /// Empties it for its next use, keeping what it holds room for.
  pub fn clear(&mut self) {
    self.emitters.clear();
  }
}
