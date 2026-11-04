use std::sync::Arc;

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::acoustics::scene::simulation_pace::SimulationPace;
use crate::control::engine::audio_engine::AudioEngine;
use crate::render::scene::scene_assets::SceneAssets;
use crate::render::scene::scene_render::SceneRender;
use crate::render::scene::scene_tick::SceneTick;
use crate::scene::host::audio_asset_source::AudioAssetSource;
use crate::scene::host::audio_scene_geometry::AudioSceneGeometry;
use xrf_audio_scene::AudioSceneFrame;
use xrf_audio_scene::AudioSceneMailbox;

/// A scene open on the control thread: where its frames come in and go back, where its sounds are read, and how it
/// plays.
pub(crate) struct ControlScene {
  mailbox: AudioSceneMailbox,
  assets: SceneAssets,
  render: SceneRender,
  /// Counts the frames taken, so the emitters a frame no longer names end.
  generation: u64,
}

impl ControlScene {
  /// A scene whose frames come through `mailbox`, its sounds read through `assets`, heard through `acoustics` and
  /// shaped by `geometry` where the host has it, simulated at `pace`.
  pub(crate) fn new(
    mailbox: AudioSceneMailbox,
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
    self.mailbox.is_closed()
  }

  /// Takes the newest frame the host sent, handing back every one it read, and plays on to the tick's mixer frame.
  pub(crate) fn tick(&mut self, tick: SceneTick<'_>) {
    let mut latest: Option<Box<AudioSceneFrame>> = None;

    while let Some(frame) = self.mailbox.take() {
      if let Some(older) = latest.replace(frame) {
        self.mailbox.give_back(older);
      }
    }

    if let Some(frame) = latest {
      self.generation += 1;
      self.render.take(&frame, tick.mixer_frame, tick.rate, self.generation);
      self.mailbox.give_back(frame);
    }

    self.render.tick(tick, &self.assets, self.generation);
  }

  /// Ends every voice it holds.
  pub(crate) fn close(&mut self, engine: &mut AudioEngine) {
    self.render.close(engine);
  }
}
