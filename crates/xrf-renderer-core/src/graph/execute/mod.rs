//! What a pass records with when the compiled graph executes: its context, and the resources resolved for the frame.

mod compute_context;
mod encoder_context;
mod executed_graph;
mod executed_group;
mod graph_bindings;
mod graph_pass_scope;
mod graph_resolved_texture;
mod graph_resources;
mod graph_runtime;
mod pass_marker;
mod raster_context;

pub use compute_context::ComputeContext;
pub use encoder_context::EncoderContext;
pub use executed_graph::ExecutedGraph;
pub use executed_group::ExecutedGroup;
pub use graph_bindings::GraphBindings;
pub(crate) use graph_pass_scope::GraphPassScope;
pub use graph_resolved_texture::GraphResolvedTexture;
pub(crate) use graph_resources::GraphResources;
pub use graph_runtime::GraphRuntime;
pub use pass_marker::PassMarker;
pub use raster_context::RasterContext;
