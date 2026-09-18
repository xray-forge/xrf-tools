//! Sound files the engine plays, and the environment presets it mixes them in.

pub(crate) mod sound_environment;
pub(crate) mod sound_environment_file;
#[cfg(test)]
mod tests;

pub(crate) mod sound_file;
pub(crate) mod sound_file_metadata;
pub(crate) mod sound_file_vorbis;

pub use crate::sound_file::SoundFile;
pub use crate::sound_file_metadata::{SoundMetadata, XRaySoundCommentVersion, XRaySoundParameters};

pub use crate::sound_environment::*;
pub use crate::sound_environment_file::*;
