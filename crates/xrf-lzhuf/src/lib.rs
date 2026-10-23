#![doc = include_str!("../README.md")]

mod bit_reader;
mod bit_writer;
mod decode;
mod encode;
mod lzhuf_constants;
mod match_distance;
mod ring_buffer;
#[cfg(test)]
mod tests;
mod tree;

pub use crate::decode::lzhuf_decode::{decompress, decompress_into};
pub use crate::encode::lzhuf_encode::compress;
