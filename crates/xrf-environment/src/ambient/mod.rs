//! Ambients: the sounds and particles a keyframe plays around the camera, shared and per level.

pub(crate) mod ambient;
pub(crate) mod ambient_effect;
pub(crate) mod ambient_effect_key;
pub(crate) mod ambient_key;
pub(crate) mod level_ambients;
pub(crate) mod sound_channel;
pub(crate) mod sound_channel_key;

pub use crate::ambient::ambient::Ambient;
pub use crate::ambient::ambient_effect::AmbientEffect;
pub use crate::ambient::ambient_effect_key::AmbientEffectKey;
pub use crate::ambient::ambient_key::AmbientKey;
pub use crate::ambient::level_ambients::LevelAmbients;
pub use crate::ambient::sound_channel::SoundChannel;
pub use crate::ambient::sound_channel_key::SoundChannelKey;
