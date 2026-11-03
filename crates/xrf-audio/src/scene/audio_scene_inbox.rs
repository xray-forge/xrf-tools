use rtrb::{Consumer, Producer, RingBuffer};

use crate::scene::audio_scene_frame::AudioSceneFrame;

/// The host's end of a scene on the control thread: frames go in through one ring and come back through another to be
/// filled again, so a warm scene allocates nothing. Dropping it closes the scene, its voices fading out.
pub struct AudioSceneInbox {
  frames: Producer<Box<AudioSceneFrame>>,
  spent: Consumer<Box<AudioSceneFrame>>,
}

/// The control thread's end.
pub(crate) struct SceneMailbox {
  pub(crate) frames: Consumer<Box<AudioSceneFrame>>,
  pub(crate) spent: Producer<Box<AudioSceneFrame>>,
}

impl AudioSceneInbox {
  /// Frames that wait for the control thread; one more is refused.
  const FRAMES_COUNT: usize = 4;

  /// Frames on their way back; the control thread keeps one more for itself.
  const SPENT_COUNT: usize = Self::FRAMES_COUNT + 2;

  pub(crate) fn open() -> (Self, SceneMailbox) {
    let (frames, received) = RingBuffer::new(Self::FRAMES_COUNT);
    let (returned, spent) = RingBuffer::new(Self::SPENT_COUNT);

    (
      Self { frames, spent },
      SceneMailbox {
        frames: received,
        spent: returned,
      },
    )
  }

  /// Sends the scene's latest frame, answering it back where the control thread has not taken the ones before, to be
  /// filled again next time.
  ///
  /// # Errors
  ///
  /// Returns the frame when the ring is full.
  pub fn send(&mut self, frame: Box<AudioSceneFrame>) -> Result<(), Box<AudioSceneFrame>> {
    self.frames.push(frame).map_err(|rtrb::PushError::Full(frame)| frame)
  }

  /// A frame the control thread is done with.
  pub fn take_spent(&mut self) -> Option<Box<AudioSceneFrame>> {
    self.spent.pop().ok()
  }

  /// Whether the control thread has let go of the scene, as it does when it shuts down.
  pub fn is_closed(&self) -> bool {
    self.frames.is_abandoned()
  }
}
