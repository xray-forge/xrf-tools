use crate::ambient::sound_channel_key::SoundChannelKey;
use crate::section::EnvironmentSection;

/// A `sound_channels.ltx` section: one set of sounds an ambient plays around the camera.
pub type SoundChannel = EnvironmentSection<SoundChannelKey>;
