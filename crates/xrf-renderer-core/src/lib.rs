#![doc = include_str!("../README.md")]

mod alloc;
mod graph;

#[cfg(test)]
mod tests;

pub use crate::alloc::{Span, SpanAllocator, SpanBuffer, SpanWrites, UploadRing, UploadSlice};
pub use crate::graph::{
  CompiledGraph, ComputeContext, ComputePassBuilder, EncodeGroup, EncoderContext, EncoderPassBuilder, FrameGraph,
  GraphBindings, GraphBuffer, GraphBufferAccess, GraphBufferDescriptor, GraphColorAttachment, GraphCompileOptions,
  GraphDepthAttachment, GraphPassKind, GraphPassReport, GraphReport, GraphResolvedTexture, GraphTexture,
  GraphTextureAccess, GraphTextureDescriptor, GraphTransientReport, RasterContext, RasterPassBuilder,
  TransientBufferKey, TransientPool, TransientSlot, TransientTextureKey,
};
