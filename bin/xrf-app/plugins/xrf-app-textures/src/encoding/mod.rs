//! The candidate formats a texture could be written in, weighed against it, and the encodes a comparison keeps until
//! one is saved over the texture they were made of.

pub(crate) mod commands;
mod comparison;
mod report;
mod request;
pub(crate) mod routes;

pub use comparison::{TextureComparison, TextureComparisons};
pub use report::{
  TextureEncodingComparison, TextureEncodingCurrent, TextureEncodingReport, TextureFormatSupport,
  TextureRendererSupport,
};
pub use request::TexturesCompareRequest;
