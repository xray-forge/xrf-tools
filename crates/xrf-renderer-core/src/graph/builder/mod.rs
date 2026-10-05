//! The builders a frame graph's passes are declared through, one per kind of pass.

mod compute_pass_builder;
mod encoder_pass_builder;
mod graph_pass_declaration;
mod raster_pass_builder;

pub use compute_pass_builder::ComputePassBuilder;
pub use encoder_pass_builder::EncoderPassBuilder;
pub use raster_pass_builder::RasterPassBuilder;
