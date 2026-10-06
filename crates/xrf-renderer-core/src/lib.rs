#![doc = include_str!("../README.md")]

// The derives name this crate by its path, which resolves inside it too.
extern crate self as xrf_renderer_core;

mod alloc;
mod graph;
mod param;
mod pipeline;
mod shader;

#[cfg(test)]
mod tests;

pub use crate::alloc::{Span, SpanAllocator, SpanBuffer, SpanWrites, UploadRing, UploadSlice};
pub use crate::graph::{
  CompiledGraph, ComputeContext, ComputePassBuilder, EncodeGroup, EncoderContext, EncoderPassBuilder, ExecutedGraph,
  ExecutedGroup, FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphBufferDescriptor,
  GraphColorAttachment, GraphCompileOptions, GraphDepthAttachment, GraphPassKind, GraphPassReport, GraphPassTime,
  GraphReport, GraphResolvedTexture, GraphRuntime, GraphTexture, GraphTextureAccess, GraphTextureDescriptor,
  GraphTimer, GraphTransientReport, RasterContext, RasterPassBuilder, TransientBufferKey, TransientPool, TransientSlot,
  TransientTextureKey,
};
pub use crate::param::{
  BindGroupCache, PassBinding, PassBindingLayout, PassParameters, PassResources, StorageArray, StorageArrayMut,
  StorageField, UniformBinding, UniformField,
};
pub use crate::pipeline::{
  ComputePipelineDescription, PipelineCache, PipelineConstants, RenderPipelineDescription, ShaderOverride,
  ShaderPermutation, ShaderSource, VertexLayout,
};
pub use crate::shader::{
  GeneratedShaderFile, ShaderAddressSpace, ShaderDeclarations, ShaderLayoutVerifier, ShaderMember, ShaderStruct,
  ShaderType, max_of, round_up,
};
pub use wgpu;
pub use xrf_renderer_derive::{PassParameters, ShaderPermutation, ShaderStruct};
