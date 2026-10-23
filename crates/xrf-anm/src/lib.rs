#![doc = include_str!("../README.md")]

pub(crate) mod anm_channel;
pub(crate) mod anm_file;

#[cfg(test)]
mod tests;

pub use crate::anm_channel::AnmChannel;
pub use crate::anm_file::AnmFile;
