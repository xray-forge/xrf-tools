use std::sync::Arc;

use xrf_audio::{AudioAssetSource, AudioSceneFrame, AudioSceneGeometry, AudioSceneInbox};
use xrf_world::{WorldSoundFrame, WorldSoundSink};

use crate::viewer::burst::{BurstSoundFrame, BurstSoundTee};

/// A shown level's sound on its way from the world to the audio's control thread: each frame the world fills is the
/// audio's own, handed to the scene's inbox as it is, and every frame the control thread hands back goes back to the
/// world to fill again. A burst's frames are sent on through the tee, and silenced on the device.
pub(crate) struct LevelSoundSink {
  inbox: AudioSceneInbox,
  tee: BurstSoundTee,
  assets: Arc<dyn AudioAssetSource>,
  geometry: Arc<dyn AudioSceneGeometry>,
  /// Frames the inbox had no room for, back to the world first; the next frame carries everything again.
  #[expect(
    clippy::vec_box,
    reason = "boxed as the inbox's rings carry them, so a frame moves between threads by its pointer"
  )]
  refused: Vec<Box<AudioSceneFrame>>,
}

impl LevelSoundSink {
  /// A sink into `inbox`, the scene's sounds read through `assets` and its walls from `geometry`, a burst's frames sent
  /// on through `tee`.
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
