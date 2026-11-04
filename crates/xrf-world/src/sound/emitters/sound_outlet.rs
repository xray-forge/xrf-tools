use std::time::Duration;

use xrf_audio_scene::AudioSceneFrame;

use crate::host::sound::sink::world_sound_frame::WorldSoundFrame;
use crate::host::sound::sink::world_sound_sink::WorldSoundSink;

/// How often a scene's sound is sent at most: the audio follows the level's clock between frames, so a faster frame
/// rate gains nothing.
const INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / 60);

/// Where a scene's sound frames leave the world: its sink, and the frames it handed back, filled again rather than
/// allocated, at most every [`INTERVAL`] of the level's clock.
pub struct SoundOutlet {
  sink: Box<dyn WorldSoundSink>,
  /// Frames to fill before the sink has handed any back.
  // Boxed as the audio's rings carry them, so a frame moves between the threads by its pointer.
  #[allow(clippy::vec_box)]
  spare: Vec<Box<AudioSceneFrame>>,
  sent_at: Option<Duration>,
}

impl SoundOutlet {
  pub fn new(sink: Box<dyn WorldSoundSink>) -> Self {
    Self {
      sink,
      spare: Vec::new(),
      sent_at: None,
    }
  }

  /// Whether a frame is due at `now`, the level's clock.
  pub fn is_due(&self, now: Duration) -> bool {
    self.sent_at.is_none_or(|at| now < at || now - at >= INTERVAL)
  }

  /// Sends the next frame whenever it comes, as on the scene's first.
  pub fn restart(&mut self) {
    self.sent_at = None;
  }

  /// Sends a frame `fill` fills at `now`, out of those handed back where there are any, emptied of what they held.
  pub fn send(&mut self, now: Duration, fill: impl FnOnce(&mut WorldSoundFrame)) {
    while let Some(spent) = self.sink.take_spent() {
      self.spare.push(spent);
    }

    let mut frame: WorldSoundFrame = WorldSoundFrame {
      sound: self.spare.pop().unwrap_or_default(),
      is_burst: false,
    };

    frame.sound.emitters.clear();
    fill(&mut frame);
    self.sink.send(frame);
    self.sent_at = Some(now);
  }
}
