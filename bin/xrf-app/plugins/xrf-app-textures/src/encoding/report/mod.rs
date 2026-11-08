//! What a comparison answers: each candidate weighed, the texture as it stands, who reads which.

mod texture_encoding_comparison;
mod texture_encoding_current;
mod texture_encoding_report;
mod texture_format_support;
mod texture_renderer_support;

pub(crate) use texture_encoding_comparison::TextureEncodingComparison;
pub(crate) use texture_encoding_current::TextureEncodingCurrent;
pub(crate) use texture_encoding_report::TextureEncodingReport;
pub(crate) use texture_format_support::TextureFormatSupport;
pub(crate) use texture_renderer_support::TextureRendererSupport;
