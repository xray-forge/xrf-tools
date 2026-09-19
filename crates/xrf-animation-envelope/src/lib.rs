//! Keyframe envelopes, the `xrCore/Animation` primitives shared by object motion and post-process effects.

pub(crate) mod animation_envelope;
pub(crate) mod animation_interpolation;
pub(crate) mod animation_key;

pub use crate::animation_envelope::AnimationEnvelope;
pub use crate::animation_interpolation::AnimationInterpolation;
pub use crate::animation_key::AnimationKey;
