use std::sync::Arc;

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::acoustics::scene::simulation_pace::SimulationPace;
use crate::control::audio_engine::AudioEngine;
use crate::host::audio_asset_source::AudioAssetSource;
use crate::host::audio_scene_geometry::AudioSceneGeometry;
use crate::render::scene::scene_assets::SceneAssets;
use crate::render::scene::scene_render::SceneRender;
use crate::render::scene::scene_tick::SceneTick;
use crate::scene::audio_scene_frame::AudioSceneFrame;
use crate::scene::inbox::scene_mailbox::SceneMailbox;

/// A scene open on the control thread: where its frames come in and go back, where its sounds are read, and how it
/// plays.
pub(crate) struct ControlScene {
  mailbox: SceneMailbox,
  assets: SceneAssets,
  render: SceneRender,
  /// Counts the frames taken, so the emitters a frame no longer names end.
  generation: u64,
}

impl ControlScene {
  /// A scene whose frames come through `mailbox`, its sounds read through `assets`, heard through `acoustics` and
  /// shaped by `geometry` where the host has it, simulated at `pace`.
  pub(crate) fn new(
    mailbox: SceneMailbox,
    assets: Arc<dyn AudioAssetSource>,
    acoustics: &AudioAcoustics,
    geometry: Option<Arc<dyn AudioSceneGeometry>>,
    pace: SimulationPace,
  ) -> Self {
    Self {
      mailbox,
      assets: SceneAssets::new(assets),
      render: SceneRender::new(acoustics, geometry, pace),
      generation: 0,
    }
  }

  /// The host's name for it, as its last frame gave it.
  pub(crate) fn get_scene(&self) -> u32 {
    self.render.get_scene()
  }

  pub(crate) fn get_render(&self) -> &SceneRender {
    &self.render
  }

  /// Whether the host has let go of it and sent nothing since.
  pub(crate) fn is_closed(&self) -> bool {
    self.mailbox.frames.is_abandoned() && self.mailbox.frames.is_empty()
  }

  /// Takes the newest frame the host sent, handing back every one it read, and plays on to the tick's mixer frame.
  pub(crate) fn tick(&mut self, tick: SceneTick<'_>) {
    let mut latest: Option<Box<AudioSceneFrame>> = None;

    while let Ok(frame) = self.mailbox.frames.pop() {
      if let Some(older) = latest.replace(frame) {
        self.give_back(older);
      }
    }

    if let Some(frame) = latest {
      self.generation += 1;
      self.render.take(&frame, tick.mixer_frame, tick.rate, self.generation);
      self.give_back(frame);
    }

    self.render.tick(tick, &self.assets, self.generation);
  }

  /// Ends every voice it holds.
  pub(crate) fn close(&mut self, engine: &mut AudioEngine) {
    self.render.close(engine);
  }

  /// Hands a frame back to the host to fill again; one the host has no room for is dropped here.
  fn give_back(&mut self, frame: Box<AudioSceneFrame>) {
    let _ = self.mailbox.spent.push(frame);
  }
}
