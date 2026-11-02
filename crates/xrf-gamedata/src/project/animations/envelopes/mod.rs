//! The files of animation envelopes the game plays from `anims\`: object motions and post-process effects.

pub(crate) mod anm_envelope_animation;
pub(crate) mod envelope_animation;
pub(crate) mod envelope_animations_verification_result;
pub(crate) mod envelope_animations_verifier;
pub(crate) mod ppe_envelope_animation;

#[cfg(test)]
mod tests;
