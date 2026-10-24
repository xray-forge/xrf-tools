//! The candidate formats a texture could be written in, weighed against it, and the encodes a comparison keeps until
//! one is saved over the texture they were made of.

pub(crate) mod commands;
pub(crate) mod routes;
mod texture_comparison;
mod texture_comparisons;
mod texture_encoding_comparison;
mod texture_encoding_current;
mod texture_encoding_report;
mod texture_format_support;
mod texture_renderer_support;
mod textures_compare_request;

pub use texture_comparison::TextureComparison;
pub use texture_comparisons::TextureComparisons;
pub use texture_encoding_comparison::TextureEncodingComparison;
pub use texture_encoding_current::TextureEncodingCurrent;
pub use texture_encoding_report::TextureEncodingReport;
pub use texture_format_support::TextureFormatSupport;
pub use texture_renderer_support::TextureRendererSupport;
pub use textures_compare_request::TexturesCompareRequest;
