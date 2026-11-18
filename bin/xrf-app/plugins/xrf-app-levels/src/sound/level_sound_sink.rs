use std::sync::Arc;

use xrf_audio::{AudioAssetSource, AudioSceneFrame, AudioSceneGeometry, AudioSceneInbox};
use xrf_world::{WorldSoundFrame, WorldSoundSink};

use crate::viewer::burst::{BurstSoundFrame, BurstSoundTee};

/// Carries a shown level's sound from the world to the audio's control thread. Frames are the audio's own and pass to
/// the scene's inbox as filled; frames handed back return to the world for refilling. During a burst, frames also go
/// to the tee and are silenced on the device.
pub(crate) struct LevelSoundSink {
  inbox: AudioSceneInbox,
  tee: BurstSoundTee,
  assets: Arc<dyn AudioAssetSource>,
  geometry: Arc<dyn AudioSceneGeometry>,
  /// Frames the inbox had no room for, returned to the world first; the next frame carries everything again.
  #[expect(
    clippy::vec_box,
    reason = "boxed as the inbox's rings carry them, so a frame moves between threads by its pointer"
  )]
  refused: Vec<Box<AudioSceneFrame>>,
}

impl LevelSoundSink {
  /// A sink into `inbox`, reading sounds through `assets` and walls from `geometry`, copying burst frames to `tee`.
  pub(crate) fn new(
    inbox: AudioSceneInbox,
    tee: BurstSoundTee,
    assets: Arc<dyn AudioAssetSource>,
    geometry: Arc<dyn AudioSceneGeometry>,
  ) -> Self {
    Self {
      inbox,
      tee,
      assets,
      geometry,
      refused: Vec::new(),
    }
  }
}

impl WorldSoundSink for LevelSoundSink {
  fn send(&mut self, frame: WorldSoundFrame) {
    let WorldSoundFrame { mut sound, is_burst } = frame;

    if is_burst {
      if self.tee.is_armed() {
        self.tee.send(BurstSoundFrame {
          frame: sound.clone(),
          assets: Arc::clone(&self.assets),
          geometry: Arc::clone(&self.geometry),
        });
      }

      sound.is_audible = false;
    }

    if let Err(refused) = self.inbox.send(sound) {
      self.refused.push(refused);
    }
  }

  fn take_spent(&mut self) -> Option<Box<AudioSceneFrame>> {
    self.refused.pop().or_else(|| self.inbox.take_spent())
  }
}
