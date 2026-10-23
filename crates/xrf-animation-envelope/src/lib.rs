#![doc = include_str!("../README.md")]

pub(crate) mod animation_behavior;
pub(crate) mod animation_envelope;
pub(crate) mod animation_envelope_evaluation;
pub(crate) mod animation_interpolation;
pub(crate) mod animation_key;
pub(crate) mod animation_shape;

pub use crate::animation_behavior::AnimationBehavior;
pub use crate::animation_envelope::AnimationEnvelope;
pub use crate::animation_interpolation::AnimationInterpolation;
pub use crate::animation_key::AnimationKey;
pub use crate::animation_shape::AnimationShape;
