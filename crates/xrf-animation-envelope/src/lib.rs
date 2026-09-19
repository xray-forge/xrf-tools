#![doc = include_str!("../README.md")]

pub(crate) mod animation_envelope;
pub(crate) mod animation_interpolation;
pub(crate) mod animation_key;

pub use crate::animation_envelope::AnimationEnvelope;
pub use crate::animation_interpolation::AnimationInterpolation;
pub use crate::animation_key::AnimationKey;
