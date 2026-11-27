use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::host::sound::world_sound::WorldSound;
use crate::host::sound::world_sound_cue::WorldSoundCue;
use crate::host::world_level_source::WorldLevelSource;
use crate::sound::level::queued_cue::QueuedCue;

/// Where a host plays its one-off sounds in a level the world shows, as a game plays its actor's footsteps: opened once
/// from the world (`World::open_sound_cues`), then played into from the host's own thread without the world, so a cue
/// never waits on the frame the render thread runs the world for. Each cue's sound is read from the level's source the
/// first time it is asked for, on the asking thread, and the cue is queued for the level's sound, which starts it at
/// its next frame.
///
/// It plays into the level it was opened for, read from that level's source as it stood then; a host showing another
/// level opens another.
pub struct WorldSoundCues {
  source: Arc<dyn WorldLevelSource>,
  /// Each cue's sound by its reference, read once.
  sounds: HashMap<String, WorldSound>,
  queue: Sender<QueuedCue>,
}

impl WorldSoundCues {
  /// The cues of the level `source` gives, queued on `queue` for its sound.
  pub(crate) fn new(source: Arc<dyn WorldLevelSource>, queue: Sender<QueuedCue>) -> Self {
    Self {
      source,
      sounds: HashMap::new(),
      queue,
    }
  }

  /// Plays `cue` at the level's next sound frame; one whose file is missing plays nothing, and neither does one played
  /// after the world let the level go.
  pub fn play(&mut self, cue: WorldSoundCue) {
    let sound: WorldSound = match self.sounds.get(&cue.reference) {
      Some(sound) => sound.clone(),
      None => {
        let sound: WorldSound = self.source.read_sound(&cue.reference);

        self.sounds.insert(cue.reference.clone(), sound.clone());

        sound
      }
    };

    // The level is gone once nothing takes its cues; a sound for it has nowhere to play.
    let _ = self.queue.send(QueuedCue { cue, sound });
  }
}
