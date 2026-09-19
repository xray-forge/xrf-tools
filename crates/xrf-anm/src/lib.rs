#![doc = include_str!("../README.md")]

pub(crate) mod anm_constants;
pub(crate) mod anm_file;

#[cfg(test)]
mod tests;

pub use crate::anm_constants::{ANM_CHANNELS, ANM_DEFAULT_FPS};
pub use crate::anm_file::AnmFile;
