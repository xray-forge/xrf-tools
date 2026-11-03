use std::sync::Arc;

use crate::control::audio_engine::AudioEngine;
use crate::host::audio_asset_source::AudioAssetSource;
use crate::render::scene_render::SceneRender;
use crate::render::sound_bank::SoundBank;
use crate::render::stream_launcher::StreamLauncher;
use crate::scene::audio_scene_frame::AudioSceneFrame;
use crate::scene::audio_scene_inbox::SceneMailbox;

/// A scene open on the control thread: where its frames come in and go back, where its sounds are read, and how it
/// plays.
pub(crate) struct ControlScene {
  mailbox: SceneMailbox,
  scope: Arc<str>,
  assets: Arc<dyn AudioAssetSource>,
  render: SceneRender,
  /// Counts the frames taken, so the emitters a frame no longer names end.
  generation: u64,
}

impl ControlScene {
  pub(crate) fn new(mailbox: SceneMailbox, assets: Arc<dyn AudioAssetSource>) -> Self {
    Self {
      mailbox,
      scope: Arc::from(assets.get_sound_scope()),
      assets,
      render: SceneRender::new(),
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

  /// Takes the newest frame the host sent, handing back every one it read, and plays on to `mixer_frame` of `rate`.
  pub(crate) fn tick(
    &mut self,
    engine: &mut AudioEngine,
    bank: &mut SoundBank,
    streams: &mut StreamLauncher,
    mixer_frame: u64,
    rate: u32,
  ) {
    let mut latest: Option<Box<AudioSceneFrame>> = None;

    while let Ok(frame) = self.mailbox.frames.pop() {
      if let Some(older) = latest.replace(frame) {
        self.give_back(older);
      }
    }

    if let Some(frame) = latest {
      self.generation += 1;
      self.render.take(&frame, mixer_frame, rate, self.generation);
      self.give_back(frame);
    }

    self.render.tick(
      engine,
      bank,
      streams,
      (&self.scope, &self.assets),
      mixer_frame,
      rate,
      self.generation,
    );
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
