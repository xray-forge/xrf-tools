#![doc = include_str!("../README.md")]

pub(crate) mod sound_environment;
pub(crate) mod sound_environment_file;
#[cfg(test)]
mod tests;

pub(crate) mod sound_file;
pub(crate) mod sound_file_vorbis;
pub(crate) mod sound_metadata;
pub(crate) mod xray_sound_comment_version;
pub(crate) mod xray_sound_parameters;

pub use crate::sound_environment::*;
pub use crate::sound_environment_file::*;
pub use crate::sound_file::SoundFile;
pub use crate::sound_metadata::SoundMetadata;
pub use crate::xray_sound_comment_version::XRaySoundCommentVersion;
pub use crate::xray_sound_parameters::XRaySoundParameters;
