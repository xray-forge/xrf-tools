//! `.anm` object motion: an envelope per channel over a fixed frame range.

pub(crate) mod anm_file;

#[cfg(test)]
mod tests;

pub use crate::anm_file::{ANM_CHANNELS, ANM_DEFAULT_FPS, AnmFile};
