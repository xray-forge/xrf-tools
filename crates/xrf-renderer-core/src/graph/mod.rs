//! The frame graph: a frame's passes declared with what they read and write, compiled, then executed.

mod access;
mod builder;
mod compile;
mod execute;
mod frame_graph;
mod pool;
mod record;
mod report;
mod resource;
mod timing;

#[cfg(test)]
mod tests;

pub use access::{GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphTextureAccess};
pub use builder::{ComputePassBuilder, EncoderPassBuilder, RasterPassBuilder};
pub use compile::{CompiledGraph, EncodeGroup, GraphCompileOptions, TransientSlot};
pub use execute::{
  ComputeContext, EncoderContext, ExecutedGraph, ExecutedGroup, GraphBindings, GraphResolvedTexture, GraphRuntime,
  PassMarker, RasterContext,
};
pub use frame_graph::FrameGraph;
pub use pool::{TransientBufferKey, TransientPool, TransientTextureKey};
pub use report::{GraphPassKind, GraphPassReport, GraphReport, GraphTransientReport};
pub use resource::{GraphBuffer, GraphBufferDescriptor, GraphTexture, GraphTextureDescriptor};
pub use timing::{GraphPassTime, GraphTimer};
