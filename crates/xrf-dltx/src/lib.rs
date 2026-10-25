#![doc = include_str!("../README.md")]

pub(crate) mod discovery;
pub(crate) mod dltx_dialect;
pub(crate) mod load;
pub(crate) mod resolve;

pub use crate::dltx_dialect::DltxDialect;

#[cfg(test)]
mod tests;
