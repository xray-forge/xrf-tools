# xrf-renderer-core

The renderer's core: what any renderer built on wgpu needs, and nothing about what it draws. It knows no scene, view or
X-Ray format; `xrf-renderer` builds the level renderer on it.

## The frame graph

A frame is declared as passes, in the order they run, each saying what it reads and writes. Compiling the graph culls
the passes whose effects nothing reads, gives every transient a slot in a pool shared with transients whose lifetimes do
not overlap, draws consecutive raster passes into the same attachments in one render pass, and cuts the frame into
encode groups. Executing it records each group into a command buffer of its own.

```rust
use xrf_renderer_core::{
  FrameGraph, GraphBufferAccess, GraphBufferDescriptor, GraphColorAttachment, GraphCompileOptions, GraphTextureAccess,
  GraphTextureDescriptor,
};

let mut graph = FrameGraph::new();
let scene = graph.create_texture(GraphTextureDescriptor::new_2d("scene", 1920, 1080, wgpu::TextureFormat::Rgba16Float));
let unused = graph.create_texture(GraphTextureDescriptor::new_2d("unused", 64, 64, wgpu::TextureFormat::Rgba8Unorm));
let output = graph.import_buffer(GraphBufferDescriptor::new("output", 1920 * 1080 * 8));

graph
  .add_raster_pass("opaque")
  .color(GraphColorAttachment::new(scene, wgpu::LoadOp::Clear(wgpu::Color::BLACK)))
  .record(|context| {
    let _pass = context.get_pass();
  });
graph
  .add_raster_pass("blended")
  .color(GraphColorAttachment::new(scene, wgpu::LoadOp::Load))
  .record(|_| {});
graph
  .add_raster_pass("unused")
  .color(GraphColorAttachment::new(unused, wgpu::LoadOp::Clear(wgpu::Color::BLACK)))
  .record(|_| {});
graph
  .add_encoder_pass("copy")
  .texture(scene, GraphTextureAccess::CopySource)
  .buffer(output, GraphBufferAccess::CopyDestination)
  .record(|_| {});

let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();

// Nothing reads "unused", and "blended" loads what "opaque" drew into the same attachment.
assert_eq!(compiled.list_pass_names(), ["opaque", "blended", "copy"]);
assert_eq!(compiled.get_culled(), ["unused"]);
assert_eq!(compiled.get_render_pass_count(), 1);
```

- **Transients and imports.** A transient is made for the frame from the `TransientPool`, which keeps textures and
  buffers across frames by what makes them interchangeable (`TransientTextureKey`: everything but the label, plus the
  usage) and frees one left unused for `TransientPool::MAX_IDLE_FRAMES`. An import comes from outside the frame and is
  bound when the graph executes (`GraphBindings`); writing one is an effect the graph keeps. A transient's usage is not
  declared: the graph gathers it from the accesses.
- **What survives.** Walking back from the last pass, a pass lives when it is kept (`keep`, for an effect the graph
  cannot see, as a readback has), is a bridge, writes an import, or writes something a living pass after it reads.
- **What is refused.** A transient read before anything writes it (a loaded attachment counts as a read), a raster pass
  with no attachment, attachments of different sizes (measured at their mip), an attachment written otherwise than by
  drawing, or named twice. An import unbound, or bound to a resource of another size, format, mip count or too little
  usage, is refused when the graph executes.
- **Merging.** A raster pass joins the render pass before it when it is in the same encode group, targets the same
  attachments, loads every one of them, and samples none.
- **Scope.** A pass records through its context and reaches only the resources it declared; debug builds assert it.
- **Bridges.** An encoder pass marked `bridge` is one recorded as before the graph, opening its own render and compute
  passes. Its accesses are declared by hand, it is never culled, and the report names it so what is left to convert
  stays visible.
- **Diagnostics.** `GraphCompileOptions` turns culling, pooling, merging and grouping off one at a time, or all at once
  (`serial`), to bisect a difference in a capture. `CompiledGraph::describe` reports the passes, the render passes and
  groups they fall in, the culled, and the transients with their slots and sizes; it serializes to JSON.

## Allocators

- **`SpanBuffer`** holds what persists and changes: a GPU buffer of fixed-size elements that ranges (`Span`) are taken
  from and given back to, so a scene's contents can be removed as well as added. `SpanAllocator` is its bookkeeping,
  with no GPU: each growth adds a region of its own `offset-allocator` (O(1), binned, cannot grow in place), so a held
  span keeps its offset. Growing copies the buffer into one at least twice the size and submits the copy at once;
  `get_generation` changes so anything bound to the old buffer is rebuilt. A new region is at least twice the request,
  because the allocator rounds a request up to its size bin and a region exactly the request's size may not hold it.
  Writes are queued and `flush` uploads them as contiguous runs (`SpanWrites`), a later write to the same bytes winning.
  The stride is a multiple of four bytes, since copies move whole words.
- **`UploadRing`** holds what one frame needs: values pushed during the frame are packed at offsets aligned for both
  uniform and storage dynamic offsets (`UploadSlice`) and uploaded in one write at `flush`, which first replaces the
  buffer with a larger one if the frame outgrew it (`get_generation` again). A value is valid from that `flush` until
  the next frame's, which the queue orders after the frame that read it.
