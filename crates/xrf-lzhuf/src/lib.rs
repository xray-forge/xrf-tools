#![doc = include_str!("../README.md")]

pub(crate) mod bits;
pub(crate) mod decode;
pub(crate) mod encode;
pub(crate) mod lzhuf_constants;
pub(crate) mod match_distance;
pub(crate) mod ring_buffer;
pub(crate) mod tree;

#[cfg(test)]
mod tests;

pub use crate::decode::LzhufDecoder;
pub use crate::encode::LzhufEncoder;
