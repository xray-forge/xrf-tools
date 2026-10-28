# xrf-renderer-core

The renderer's core: what any renderer built on wgpu needs, and nothing about what it draws. It knows no scene, view or
X-Ray format. [xrf-renderer](../xrf-renderer/README.md) builds the level renderer on it, and
[xrf-world](../xrf-world/README.md) hands out scene item handles with its `ProxyAllocator`. It holds the frame graph,
the allocators and stores that keep GPU data, the shader structs, bind groups and permutations declared once in Rust
(with the derives of [xrf-renderer-derive](../xrf-renderer-derive/README.md), re-exported here), and the pipeline
cache. It re-exports `wgpu`.

## The frame graph

A frame is a `FrameGraph` of passes, declared in the order they run, each saying what it reads and writes:
`add_raster_pass` draws into attachments the graph opens a render pass on, `add_compute_pass` dispatches, and
`add_encoder_pass` records on the command encoder itself (copies, clears). Compiling culls the passes whose effects
nothing reads, gives every transient a slot in a pool shared with transients whose lifetimes do not overlap, draws
consecutive raster passes into the same attachments in one render pass, and cuts the frame into encode groups. Compiling
needs no device:

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

- **Transients and imports.** `create_texture` and `create_buffer` declare a transient, made for the frame from the
  `TransientPool`. Its usage is not declared: the graph gathers it from the accesses. `import_texture` and
  `import_buffer` declare a resource from outside the frame, bound when the graph executes; writing one is an effect
  the graph keeps. A pass is declared only by its `record` call; the builders are `#[must_use]`.
- **What survives.** Walking back from the last pass, a pass lives when it is kept (`keep`, for an effect the graph
  cannot see, as a readback has), is a bridge, writes an import, or writes something a living pass after it reads. An
  attachment the pass loads rather than clears counts as a read.
- **What compile refuses.** A transient read before anything writes it; a raster pass with no attachment, with
  attachments of different sizes (measured at their mip), drawing into a mip or layer the texture lacks, or naming an
  attachment twice; and a raster pass that also accesses its own attachment, except to sample a read-only depth.
- **Merging.** A raster pass joins the render pass before it when it is in the same encode group, targets the same
  attachments, loads every one of them, samples none, and shares no other resource with a pass of that render pass
  where either writes it, since nothing would stand between their draws.
- **Encode groups.** `begin_group` starts a group of its own encoder; groups left empty by culling are dropped.
  `begin_owner` names whose the passes after it are (`GraphOwner::Part`), for a frame drawing several views, so their
  timings are summed apart.
- **Bridges.** An encoder pass marked `bridge` is one recorded as before the graph, opening its own render and compute
  passes. Its accesses are declared by hand and the graph cannot check them; it is never culled, and the report names
  it so what is left to convert stays visible.
- **Diagnostics.** `GraphCompileOptions` turns culling, pooling, merging and grouping off one at a time, or all at once
  (`serial()`), to bisect a difference in a capture. `CompiledGraph::describe` answers a `GraphReport` of the passes,
  the render passes and groups they fall in and the textures each reads and writes, the culled, every texture's label,
  and the transients with their slots and sizes; it serializes to JSON.

## Executing a frame

`CompiledGraph::execute((device, queue), runtime, bindings)` records the frame and answers an `ExecutedGraph`: the
command buffers in order for one submit, and what each group cost the CPU (`groups`: recording, and wgpu's encoding in
`finish`; `encode`, what the executing thread waited on).

- **`GraphRuntime`** is what frames keep between them: the `TransientPool`, the `BindGroupCache`, the views of the mips
  and layers passes draw into, the `UploadRing` a frame's uniforms are pushed to (`push_uniform`), and the `GraphTimer`.
  Executing flushes the ring first, so a uniform is bound from the buffer it lies in.
- **`GraphBindings`** gives each import its real resource: `bind_texture` and `bind_buffer` for a handle declared
  earlier, or `import_view` and `import_buffer` to declare a resource as it is and bind it in one step. An import a
  surviving pass names that is unbound, or bound to a resource of another size, format, mip count or too little usage,
  fails `execute`.
- **Pooling.** The pool keeps textures and buffers across frames by what makes them interchangeable
  (`TransientTextureKey`: everything but the label, plus the usage; `TransientBufferKey`: size and usage) and frees one
  left unused for `TransientPool::MAX_IDLE_FRAMES` (30).
- **Recording.** Groups record in parallel on rayon's pool, each into a command encoder of its own. A pass reaches only
  the resources it declared, through its context (`get_texture`, `get_buffer`); debug builds assert it. An encoder pass
  may `hand_over` command buffers recorded apart, which are submitted right after what it and the passes before it
  recorded. If wgpu refuses any group, the whole frame is dropped with an error and its stamps are discarded.

## Timing

`GraphTimer` measures each pass on the GPU with no code in any pass. While enabled (`set_enabled`), the graph writes a
timestamp at the start of each encode group and after each pass, each group into its own range of the query set so the
groups still record at once; a render pass of merged passes counts as one, its passes' names joined with ` + `. Call
`request` after each submit: it maps the stamps without waiting, a few frames in flight at once. `take(owner)` answers
each of that owner's passes' mean milliseconds a frame since its last call, a pass a frame skipped counting as zero so
the passes add up to the frame; `take_tagged` answers every pass of every frame under the tag set with `set_tag`. A
device without timestamps inside encoders times nothing.

A bridge recording several stages times each through `EncoderContext::split`'s `PassMarker`: each `mark` ends a stage
named for it, and a pass that marks is timed by its marks alone.

The `tracy` feature streams CPU zones, frame marks and the timed passes as GPU zones to a Tracy capture;
`start_profiler` starts the client and `send_profiler_message` marks the timeline. Without the feature both do nothing.

## Pass parameters

A pass's bind group is declared as a struct with `#[derive(PassParameters)]` (`xrf-renderer-derive`'s README lists the
field kinds). A builder's `parameters(&p)` adds the accesses the struct makes, and the recording binds it with
`context.bind(&p)`, at the group the struct names, with its dynamic offsets; `bind_at` binds it at another index, for a
pipeline holding the same layout elsewhere. Groups the scene owns (a bindless texture array, the scene's buffers) are
bound on the context's render or compute pass directly.

- **`UniformBinding<T>`** (from `GraphRuntime::push_uniform`) binds the upload ring's buffer once and the value's offset
  as a dynamic offset, so one bind group serves every frame's uniforms. The buffer is resolved when the pass binds,
  after `execute` has flushed the ring, so a ring that grew is never bound stale. A storage field takes a graph buffer:
  `StorageArray::new_range` binds part of one, from an offset aligned to the device's storage offset alignment.
- **`BindGroupCache`** makes each struct's layout once, keyed by `PassParameters::LAYOUT_KEY` (its path); a pipeline for
  that pass takes its layout from `get_layout`, so the bind groups match it. Bind groups are kept across frames by the
  identity of what they bind (wgpu's resources compare and hash by identity), so the pooled transients a pass binds
  give the same bind group back every frame; one unused for `BindGroupCache::MAX_IDLE_FRAMES` (30) is dropped. It locks,
  for the passes recording a frame in parallel. An acceleration structure is not a resource the cache can hold, so each
  one made takes a key of its own (`AccelerationStructureBinding::next_key`).
- **`create_layout`** makes the struct's layout outside any cache; layouts of the same entries are interchangeable, so a
  pipeline made with it takes the bind groups any runtime's cache makes.
- **`ShaderBindings`** gathers the WGSL bindings of every struct whose passes draw with one module, each once, `enable`
  lines first: passes sharing a module number their bindings apart (`#[binding(N)]`) or declare a shared one alike. A
  clash is an error that adds nothing of the struct, as is a struct whose WGSL lines do not match its `BINDINGS`.

## Shared structs

A struct a shader and Rust share is declared once, in Rust, with `#[derive(ShaderStruct)]`. Its members are
`ShaderType`s: `f32`, `u32`, `i32`, glam's `Vec2`/`Vec3`/`Vec4`, their `U` and `I` forms, `Mat3A` and `Mat4`,
`ShaderAtomicU32` (`atomic<u32>`), arrays of any of these, and other shared structs. A Rust array is a WGSL `array`,
never a vector (`[u32; 4]` is `array<u32, 4>`, `UVec4` is `vec4<u32>`). glam's `Mat3` and `Vec3A` are left out because
WGSL lays them out otherwise.

- The derive computes WGSL's layout in constants and asserts every member's Rust offset and the struct's size against
  it at compile time. Padding is written out as members named with a leading underscore, which WGSL never sees; where
  Rust would pad on its own, `bytemuck::Pod` refuses the struct first.
- `ShaderStruct::get_wgsl_declaration` writes the WGSL struct; `ShaderDeclarations` gathers a shader's, each once and a
  held struct before its holder.
- `ShaderLayoutVerifier::verify::<T>(space)` is the independent check, for a test: naga parses the declaration, lays it
  out itself, and validates it bound in a `ShaderAddressSpace` (`Uniform` or `Storage`). A uniform buffer's
  sixteen-byte array stride and struct alignment are naga's to enforce, so an array of scalars passes in storage and is
  refused as a uniform.

## Permutations and pipelines

- **`ShaderPermutation`** (`#[derive(ShaderPermutation)]`) declares a shader's variants as a struct of `bool`, `u32`,
  `i32` or `f32` fields (or any `ShaderOverride`): each a WGSL `override` constant of its name in upper case, declared
  by `get_wgsl_overrides` and set from the field when a pipeline is made. `()` is the permutation of a shader with
  none. A permutation switches code, never bindings; a variant that binds otherwise is another parameter struct. A
  description takes override constants only from one (`with_permutation`), never by name.
- **`PipelineCache`** makes pipelines from hashable descriptions (`RenderPipelineDescription`,
  `ComputePipelineDescription`: module, entry points, the permutation's `PipelineConstants`, the bind group layouts by
  identity, the bytes of immediates a render layout holds, the render state, owned `VertexLayout`s) and keeps them, and
  the modules they compile, by that key. WGSL comes from a `ShaderSource`, which composes a module by name with its
  imports resolved. `warm` makes a level's pipelines ahead of the frames that draw with them. `invalidate_module` drops
  a module hot reload changed and every pipeline made from it, `invalidate_all` drops everything, and each bumps
  `get_generation`, so a holder of a pipeline knows to ask again.
- **Errors.** Every creation runs under a wgpu validation scope, so what wgpu refuses comes back as an error naming the
  module or pipeline. A shader compile error names lines of the composed text; `ShaderLines` maps each back to the
  module and line it came from, using the markers a source writes with `ShaderLines::mark`.
- **Trusted modules.** Modules are created with `create_shader_module_trusted`, with bounds checks and loop bounding
  off; division and ray queries stay guarded. This is sound only for the renderer's own WGSL: every loop must end,
  storage buffers are bound through descriptors the backends bound-check in hardware, and a texture binding array is
  indexed only by slots the renderer itself handed out. A new shader must keep to that.
- **Driver cache.** `PipelineCache::with_driver_cache(device, data)` makes pipelines with the driver's own cache where
  the device has `Features::PIPELINE_CACHE`. `get_driver_data` answers its contents behind a CRC32 checksum, to save
  and hand back at the next start; data whose checksum fails is logged and ignored, and wgpu itself refuses data of
  another adapter or driver.

## Allocators

- **`SpanBuffer`** holds what persists and changes: GPU buffers of fixed-size elements that ranges (`Span`) are taken
  from and given back to, so a scene's contents can be removed as well as added. Its lanes (`SpanLane`) are parallel
  buffers sharing one allocator, so a span addresses the same elements of each. `SpanAllocator` is the bookkeeping,
  with no GPU: each growth adds a region of its own `offset-allocator` (O(1), binned, cannot grow in place), so a held
  span keeps its offset. Growing copies each lane into one at least twice the size and submits the copy at once, and
  fails past the device's `max_buffer_size`; `get_generation` changes so anything bound to the old buffers is rebuilt.
  A new region is at least twice the request, because the allocator rounds a request up to its size bin and a region
  exactly the request's size may not hold it. Writes are queued and `flush` uploads them as contiguous runs
  (`SpanWrites`), a later write to the same bytes winning. A freed span keeps its bytes until overwritten, so `clear`
  it first where a shader walks the whole buffer. Every stride is a multiple of four bytes, since copies move whole
  words.
- **`UploadRing`** holds what one frame needs: values pushed during the frame are packed at offsets aligned for both
  uniform and storage dynamic offsets (`UploadSlice`) and uploaded in one write at `flush`, which first replaces the
  buffer with a larger one if the frame outgrew it (`get_generation` again). A value is valid from that `flush` until
  the next frame's, which the queue orders after the frame that read it.

## Scene stores

A scene's items are stores, one kind each: `ProxyStore<T>` keeps them as dense records the GPU mirrors element for
element. An add appends and answers a `ProxyHandle<T>`; a remove moves the last record into the gap, so the records
stay dense and the moved item keeps its handle. A handle is generational: once its item is gone it answers nothing,
even after its slot is reused. Every record added, changed (`get_mut`) or moved is noted; `StoreMirror::sync` uploads
those as contiguous runs into a buffer of the store's records, growing it by a GPU copy (`get_generation` again).

Where items are made ahead of the store holding them, as a world posting items to a scene, a `ProxyAllocator<T>` hands
out the handles and the store takes each item at its handle with `insert`. A store takes its handles from `add` or from
an allocator, never both.

## Generated files

`GeneratedShaderFile::sync` keeps a checked-in WGSL file written from Rust declarations (`ShaderDeclarations`, pass
parameters' `get_wgsl_bindings`, permutations' `get_wgsl_overrides`) the way the TypeScript bindings are kept: a test
syncs it, the file is rewritten under a generated header when stale (line endings aside) and the test fails once, then
passes with the file committed.

## Checks

Run `cargo test --locked -p xrf-renderer-core`. The graph's compile tests need no GPU. The execution, allocator, mirror,
pipeline and parameter tests open a wgpu device, falling back to the software adapter (WARP under D3D12); on a machine
with neither they skip with a message, or fail where `XRF_TEST_REQUIRE_GPU` is set, as Windows CI sets it.

Sources: the [frame graph](src/graph/frame_graph.rs) and its [compiler](src/graph/compile/graph_compiler.rs), the
[pipeline cache](src/pipeline/pipeline_cache.rs), the [bind group cache](src/param/bind_group_cache.rs), the
[span buffer](src/alloc/span_buffer.rs) and the [proxy store](src/store/proxy_store.rs).
