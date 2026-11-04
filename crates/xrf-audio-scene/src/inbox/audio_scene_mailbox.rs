use rtrb::{Consumer, Producer};

use crate::audio_scene_frame::AudioSceneFrame;

/// The audio's end of a scene's [`crate::AudioSceneInbox`]: the frames the host sent, and the ring they go back through
/// to be filled again.
pub struct AudioSceneMailbox {
  pub(crate) frames: Consumer<Box<AudioSceneFrame>>,
  pub(crate) spent: Producer<Box<AudioSceneFrame>>,
}

impl AudioSceneMailbox {
  /// The oldest frame the host sent and the audio has not taken, none while there is none.
  pub fn take(&mut self) -> Option<Box<AudioSceneFrame>> {
    self.frames.pop().ok()
  }

  /// Hands a frame back to the host to fill again; one the host has no room for is dropped.
  pub fn give_back(&mut self, frame: Box<AudioSceneFrame>) {
    let _ = self.spent.push(frame);
  }

  /// Whether the host has let go of the scene and nothing it sent waits.
  pub fn is_closed(&self) -> bool {
    self.frames.is_abandoned() && self.frames.is_empty()
  }
}
