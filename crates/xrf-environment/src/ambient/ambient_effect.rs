use crate::ambient::ambient_effect_key::AmbientEffectKey;
use crate::section::EnvironmentSection;

/// An `effects.ltx` section: a particle effect an ambient plays near the camera.
pub type AmbientEffect = EnvironmentSection<AmbientEffectKey>;
