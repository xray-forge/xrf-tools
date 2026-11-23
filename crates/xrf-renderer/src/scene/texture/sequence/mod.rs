//! Animated textures: a `.seq` frame list beside a texture's name, its frames shown in turn by the time as the engine's
//! `CTexture::apply_seq` binds them.

pub(crate) mod decoded_sequence;
pub(crate) mod played_sequence;
pub(crate) mod texture_sequence;
pub(crate) mod texture_sequence_timing;
