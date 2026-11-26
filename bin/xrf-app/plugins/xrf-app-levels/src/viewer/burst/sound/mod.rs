//! A burst's sound: frames teed off the live scene, played on the burst's clock and written beside its frames.

mod burst_sound;
mod burst_sound_chunk;
mod burst_sound_outcome;

pub(crate) use burst_sound::BurstSound;
pub(crate) use burst_sound_chunk::BurstSoundChunk;
pub(crate) use burst_sound_outcome::BurstSoundOutcome;
