use crate::host::sound::world_sound::WorldSound;
use crate::host::sound::world_sound_cue::WorldSoundCue;

/// A host's cue on its way to the level's sound, with its sound as the asking thread read it, so starting it reads no
/// file.
pub(crate) struct QueuedCue {
  pub cue: WorldSoundCue,
  pub sound: WorldSound,
}
