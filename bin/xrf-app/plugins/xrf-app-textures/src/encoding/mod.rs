//! The candidate formats a texture could be written in, weighed against it, and the encodes a comparison keeps until
//! one is saved over the texture they were made of.

pub(crate) mod commands;
mod comparison;
mod report;
mod request;
pub(crate) mod routes;

pub(crate) use comparison::TextureComparison;
pub use comparison::TextureComparisons;
pub(crate) use report::{
  TextureEncodingComparison, TextureEncodingCurrent, TextureEncodingReport, TextureFormatSupport,
  TextureRendererSupport,
};
pub(crate) use request::TexturesCompareRequest;
