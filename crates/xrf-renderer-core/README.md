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

## Shared structs

A struct a shader and Rust share is declared once, in Rust, with `#[derive(ShaderStruct)]` (from `xrf-renderer-derive`,
re-exported here; its README shows it). Its members are `ShaderType`s: `f32`, `u32`, `i32`, glam's `Vec2`/`Vec3`/`Vec4`,
their `U` and `I` forms, `Mat3A` and `Mat4`, arrays of any of these, and other shared structs. A Rust array is a WGSL
`array`, never a vector (`[u32; 4]` is `array<u32, 4>`, `UVec4` is `vec4<u32>`); glam's `Mat3` and `Vec3A` are left out,
because WGSL lays them out otherwise.

- The derive computes WGSL's layout in constants and asserts every member's Rust offset and the struct's size against
  it at compile time. Padding is written out as members named with a leading underscore, which WGSL never sees; where
  Rust would pad on its own, `bytemuck::Pod` refuses the struct first.
- `ShaderStruct::get_wgsl_declaration` writes the WGSL struct; `ShaderDeclarations` gathers a shader's, each once and a
  held struct before its holder.
- `ShaderLayoutVerifier::verify` is the independent check: naga parses that declaration, lays it out itself, and
  validates it bound in a `ShaderAddressSpace`. A uniform buffer's sixteen-byte array stride and struct alignment are
  naga's to enforce, so an array of scalars passes in storage and is refused as a uniform.

## Pass parameters

A pass's bind group is declared as a struct with `#[derive(PassParameters)]` (`xrf-renderer-derive`'s README lists the
field kinds). A builder's `parameters(&p)` adds the accesses the struct makes, and the recording binds it with
`context.bind(&p)`, at the group the struct names, with its dynamic offsets.

- **`BindGroupCache`** makes each struct's layout once (keyed by its path, `PassParameters::LAYOUT_KEY`); a pipeline for
  that pass takes its layout from `get_layout`, so the bind groups match it. Bind groups are kept across frames by the
  identity of what they bind (wgpu's resources compare and hash by identity), so the pooled transients a pass binds give
  the same bind group back every frame; one unused for `BindGroupCache::MAX_IDLE_FRAMES` is dropped. It locks, for the
  passes recording a frame.
- `CompiledGraph::execute` takes the cache beside the pool, and starts a frame of both.
- `UniformBinding` binds the upload ring's buffer once and the value's offset as a dynamic offset, so one bind group
  serves every frame's uniforms.
