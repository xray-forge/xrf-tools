#![doc = include_str!("../README.md")]

pub(crate) mod philox;
pub(crate) mod stream;

#[cfg(test)]
mod tests;

pub use crate::philox::philox4x64::Philox4x64;
pub use crate::stream::random_address::RandomAddress;
pub use crate::stream::random_stream::RandomStream;
