//! A burst's sound: the scene's frames a burst draws teed off the live scene, played on the burst's clock and written
//! beside its frames.

mod burst_sound;
mod burst_sound_chunk;
mod burst_sound_frame;
mod burst_sound_outcome;
mod burst_sound_tee;

pub(crate) use burst_sound::BurstSound;
pub(crate) use burst_sound_chunk::BurstSoundChunk;
pub(crate) use burst_sound_frame::BurstSoundFrame;
pub(crate) use burst_sound_outcome::BurstSoundOutcome;
pub(crate) use burst_sound_tee::BurstSoundTee;
