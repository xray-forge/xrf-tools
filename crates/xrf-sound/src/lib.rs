#![doc = include_str!("../README.md")]

pub(crate) mod environment;
pub(crate) mod sound_file;
pub(crate) mod sound_metadata;
#[cfg(test)]
mod tests;
pub(crate) mod vorbis;
pub(crate) mod xray;

pub use crate::environment::eax_environment::EaxEnvironment;
pub use crate::environment::sound_environment::SoundEnvironment;
pub use crate::environment::sound_environment_file::SoundEnvironmentFile;
pub use crate::sound_file::SoundFile;
pub use crate::sound_metadata::SoundMetadata;
pub use crate::xray::xray_sound_comment_version::XRaySoundCommentVersion;
pub use crate::xray::xray_sound_game_type::XRaySoundGameType;
pub use crate::xray::xray_sound_parameters::XRaySoundParameters;
