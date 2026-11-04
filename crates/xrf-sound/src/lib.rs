#![doc = include_str!("../README.md")]

pub(crate) mod environment;
pub(crate) mod sound_file;
pub(crate) mod sound_metadata;
pub(crate) mod vorbis;
pub(crate) mod xray;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;

#[cfg(test)]
mod tests;

pub use crate::environment::eax_environment::EaxEnvironment;
pub use crate::environment::sound_environment::SoundEnvironment;
pub use crate::environment::sound_environment_file::SoundEnvironmentFile;
pub use crate::sound_file::SoundFile;
pub use crate::sound_metadata::SoundMetadata;
pub use crate::vorbis::vorbis_decoder::VorbisDecoder;
pub use crate::xray::xray_sound_comment_version::XRaySoundCommentVersion;
pub use crate::xray::xray_sound_game_type::XRaySoundGameType;
pub use crate::xray::xray_sound_parameters::XRaySoundParameters;
