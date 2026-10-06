//! What a pass records with when the compiled graph executes: its context, and the resources resolved for the frame.

mod compute_context;
mod encoder_context;
mod graph_bindings;
mod graph_pass_scope;
mod graph_resolved_texture;
mod graph_resources;
mod raster_context;

pub use compute_context::ComputeContext;
pub use encoder_context::EncoderContext;
pub use graph_bindings::GraphBindings;
pub(crate) use graph_pass_scope::GraphPassScope;
pub use graph_resolved_texture::GraphResolvedTexture;
pub(crate) use graph_resources::GraphResources;
pub use raster_context::RasterContext;
