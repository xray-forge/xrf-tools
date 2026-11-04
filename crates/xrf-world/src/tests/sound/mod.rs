mod ambient_channels;
mod level_music;
mod level_sound;
mod rain_sound;
mod sound_outlet;
mod static_sounds;
mod zone_sounds;

use std::time::Duration;

use crate::host::sound::world_sound::WorldSound;
use crate::host::sound::world_sound_info::WorldSoundInfo;

/// A mono sound found, `seconds` long.
pub(crate) fn found(reference: &str, seconds: u64) -> WorldSound {
  WorldSound::new(
    reference,
    Some(WorldSoundInfo {
      length: Duration::from_secs(seconds),
      ..WorldSoundInfo::default()
    }),
  )
}

/// A sound whose file is missing.
pub(crate) fn missing(reference: &str) -> WorldSound {
  WorldSound::new(reference, None)
}

pub(crate) fn seconds(seconds: u64) -> Duration {
  Duration::from_secs(seconds)
}
