//! What a comparison answers: each candidate weighed, the texture as it stands, who reads which.

mod texture_encoding_comparison;
mod texture_encoding_current;
mod texture_encoding_report;
mod texture_format_support;
mod texture_renderer_support;

pub use texture_encoding_comparison::TextureEncodingComparison;
pub use texture_encoding_current::TextureEncodingCurrent;
pub use texture_encoding_report::TextureEncodingReport;
pub use texture_format_support::TextureFormatSupport;
pub use texture_renderer_support::TextureRendererSupport;
