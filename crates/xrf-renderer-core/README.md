# xrf-renderer-core

What any wgpu renderer needs, and nothing about what it draws. It knows no scene, view or X-Ray format.

It holds:

- the **frame graph**, which orders, culls, pools and times a frame's passes;
- **allocators and stores** that keep GPU data;
- **shader structs, bind groups and permutations** declared once in Rust, with the derives of
  [xrf-renderer-derive](../xrf-renderer-derive/README.md), re-exported here;
- the **pipeline cache**.

It re-exports `wgpu`. [xrf-renderer](../xrf-renderer/README.md) builds the level renderer on it, and
[xrf-world](../xrf-world/README.md) hands out scene item handles with its `ProxyAllocator`.

## The frame graph

A frame is a `FrameGraph` of passes, declared in running order, each saying what it reads and writes:

| Builder            | Records                                                 |
| ------------------ | ------------------------------------------------------- |
| `add_raster_pass`  | Draws into attachments; the graph opens the render pass |
| `add_compute_pass` | Dispatches                                              |
| `add_encoder_pass` | On the command encoder itself: copies, clears           |

Compiling the graph:

- culls passes whose effects nothing reads;
- gives every transient a slot in a pool shared with transients whose lifetimes do not overlap;
- merges consecutive raster passes into the same attachments into one render pass;
- cuts the frame into encode groups.

The graph is declared, not discovered: passes run in declaration order and are never reordered. What it culled, merged
and pooled is in its report (`CompiledGraph::describe`), with each pass's reads, writes and tags, so a renderer can check
the frame it declared. Compiling needs no device:

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

### Resources

- `create_texture` and `create_buffer` declare a **transient**, made for the frame from the `TransientPool`. Its usage
  is not declared; the graph gathers it from the accesses.
- `import_texture` and `import_buffer` declare a resource from outside the frame, bound when the graph executes.
  Writing an import is an effect the graph keeps.
- A pass exists only once `record` is called; the builders are `#[must_use]`.

### What survives culling

Walking back from the last pass, a pass survives when it:

- is kept explicitly (`keep`, for an effect the graph cannot see, such as a readback);
- is a bridge;
- writes an import;
- or writes something a surviving later pass reads. An attachment a pass loads rather than clears counts as a read.

### What compiling refuses

- A transient read before anything writes it.
- A raster pass with no attachment, with attachments of different sizes (measured at their mip), drawing into a mip or
  layer the texture lacks, or naming an attachment twice.
- A raster pass that also accesses its own attachment, except to sample a read-only depth.

### Merging and grouping

- A raster pass joins the previous render pass when it is in the same encode group, targets the same attachments, loads
  all of them, samples none, and shares no other resource with that render pass's passes where either side writes it.
- `begin_group` starts an encode group with its own encoder; groups emptied by culling are dropped.
- `begin_owner` marks which view the following passes belong to (`GraphOwner::Part`), so a frame drawing several views
  sums their timings separately.

### Tags, bridges and diagnostics

- `tag` attaches a `&'static str` to a pass in the report (`GraphPassReport::tags`) for checks over the declared frame;
  the graph itself reads none.
- A **bridge** is an encoder pass marked `bridge`, recorded as code did before the graph existed, opening its own
  render and compute passes. Its accesses are declared by hand and cannot be checked. It is never culled, and the report
  names it, so what is left to convert stays visible.
- `GraphCompileOptions` turns culling, pooling, merging and grouping off one at a time, or all at once (`serial()`), to
  bisect a difference in a capture.
- `CompiledGraph::describe` returns a `GraphReport`: the passes, their render passes and groups, the textures each
  reads and writes, the culled passes, every texture's label, and the transients with their slots and sizes. It
  serializes to JSON.

## Execute a frame

`CompiledGraph::execute((device, queue), runtime, bindings)` records the frame and returns an `ExecutedGraph`: the
command buffers, in order, for one submit, and each group's CPU cost (`groups`: recording plus wgpu's `finish`;
`encode`: what the executing thread waited for).

**`GraphRuntime`** is what persists between frames:

- the `TransientPool`, the `BindGroupCache`, and the views of the mips and layers passes draw into;
- the `UploadRing` that a frame's uniforms are pushed to (`push_uniform`). Executing flushes the ring first, so a
  uniform is bound from the buffer it is in;
- the `GraphTimer`.

**`GraphBindings`** gives each import its real resource:

- `bind_texture` and `bind_buffer` bind a handle declared earlier;
- `import_view` and `import_buffer` declare and bind in one step;
- `execute` fails if a surviving pass names an import that is unbound, or bound to a resource of another size, format
  or mip count, or with too little usage.

**Pooling.** The pool keeps textures and buffers across frames, keyed by what makes them interchangeable:

- `TransientTextureKey`: everything but the label, plus the usage;
- `TransientBufferKey`: size and usage.

One left unused for `TransientPool::MAX_IDLE_FRAMES` (30) frames is freed.

**Recording.**

- Groups record in parallel on rayon's pool, each into its own command encoder.
- A pass reaches only the resources it declared, through its context (`get_texture`, `get_buffer`); debug builds assert
  it.
- An encoder pass may `hand_over` command buffers recorded elsewhere; they are submitted right after what it and earlier
  passes recorded.
- If wgpu refuses any group, the whole frame is dropped with an error and its timestamps are discarded.

## Timing

`GraphTimer` measures each pass on the GPU without any code in the passes.

- While enabled (`set_enabled`), the graph writes a timestamp at the start of each encode group and after each pass.
  Each group writes into its own range of the query set, so groups still record in parallel.
- A render pass of merged passes counts as one; its passes' names are joined with `TimedPass::SEPARATOR` (a spaced
  plus).
- Call `request` after each submit. It maps the timestamps without waiting, with a few frames in flight.
- `take(owner)` returns each of that owner's passes' mean milliseconds per frame since the last call. A pass skipped in
  a frame counts as zero, so the passes add up to the frame.
- `take_tagged` returns every pass of every frame under the tag set with `set_tag`.
- A device without timestamps inside encoders times nothing.

A bridge recording several stages times each through `EncoderContext::split`'s `PassMarker`: each `mark` ends a stage
named after it, and a pass that marks is timed by its marks alone.

With the `tracy` feature, timed passes become GPU zones in a Tracy capture, beside the CPU zones and frame marks every
crate streams through `xrf-profiler`. Without it, the GPU zones do nothing.

## Pass parameters

A pass's bind group is a struct with `#[derive(PassParameters)]` (`xrf-renderer-derive`'s README lists the field
kinds).

- A builder's `parameters(&p)` adds the accesses the struct makes.
- The recording binds it with `context.bind(&p)`, at the group the struct names, with its dynamic offsets. `bind_at`
  binds it at another index, for a pipeline holding the same layout elsewhere.
- Groups the scene owns (a bindless texture array, the scene's buffers) are bound directly on the context's render or
  compute pass.

**`UniformBinding<T>`** (from `GraphRuntime::push_uniform`) binds the upload ring's buffer once, with the value's offset
as a dynamic offset, so one bind group serves every frame's uniforms. The buffer is resolved at bind time, after
`execute` flushed the ring, so a grown ring is never bound stale.

**Storage.** A storage field takes a graph buffer. `StorageArray::new_range` binds part of one, from an offset aligned
to the device's storage offset alignment.

**`BindGroupCache`**:

- makes each struct's layout once, keyed by `PassParameters::LAYOUT_KEY` (its path). A pipeline for that pass takes its
  layout from `get_layout`, so bind groups match it;
- keeps bind groups across frames, keyed by the identity of what they bind (wgpu resources compare by identity), so
  pooled transients give back the same bind group every frame. One unused for `BindGroupCache::MAX_IDLE_FRAMES` (30)
  frames is dropped;
- locks, for passes recording in parallel;
- cannot hold an acceleration structure as a resource, so each one made takes its own key
  (`AccelerationStructureBinding::next_key`).

**`create_layout`** makes a struct's layout outside any cache. Layouts of the same entries are interchangeable, so a
pipeline made with one accepts bind groups from any runtime's cache.

**`ShaderBindings`** gathers the WGSL bindings of every struct whose passes draw with one module, each once, `enable`
lines first. Passes sharing a module number their bindings apart (`#[binding(N)]`) or declare a shared one identically.
A clash is an error that adds nothing from the struct, as is a struct whose WGSL lines do not match its `BINDINGS`.

## Shared structs

A struct a shader and Rust share is declared once, in Rust, with `#[derive(ShaderStruct)]`.

Members are `ShaderType`s:

- `f32`, `u32`, `i32`;
- glam's `Vec2`, `Vec3`, `Vec4` and their `U` and `I` forms, `Mat3A`, `Mat4`;
- `ShaderAtomicU32` (`atomic<u32>`);
- arrays of any of these, and other shared structs.

A Rust array is a WGSL `array`, never a vector: `[u32; 4]` is `array<u32, 4>`, `UVec4` is `vec4<u32>`. glam's `Mat3`
and `Vec3A` are left out because WGSL lays them out differently.

- The derive computes WGSL's layout in constants and asserts every member's Rust offset and the struct's size against
  it at compile time.
- Padding is written out as members named with a leading underscore, which WGSL never sees. Where Rust would pad on its
  own, `bytemuck::Pod` refuses the struct first.
- `ShaderStruct::get_wgsl_declaration` writes the WGSL struct. `ShaderDeclarations` gathers a shader's structs, each
  once, a contained struct before its container.
- `ShaderLayoutVerifier::verify::<T>(space)` is the independent check, for tests: naga parses the declaration, lays it
  out, and validates it bound in a `ShaderAddressSpace` (`Uniform` or `Storage`). naga enforces a uniform buffer's
  16-byte array stride and struct alignment, so an array of scalars passes in storage and fails as a uniform.

## Permutations and pipelines

**`ShaderPermutation`** (`#[derive(ShaderPermutation)]`) declares a shader's variants as a struct of `bool`, `u32`,
`i32` or `f32` fields (or any `ShaderOverride`).

- Each field is a WGSL `override` constant named in upper case, declared by `get_wgsl_overrides` and set from the field
  when a pipeline is made. `()` is the permutation of a shader with none.
- A permutation switches code, never bindings; a variant that binds differently is another parameter struct.
- A pipeline description takes override constants only from a permutation (`with_permutation`), never by name.

**`PipelineCache`** makes pipelines from hashable descriptions (`RenderPipelineDescription`,
`ComputePipelineDescription`) and keeps them, and the modules they compile, by that key. A description holds the
module, entry points, the permutation's `PipelineConstants`, bind group layouts by identity, the bytes of immediates a
render layout holds, the render state, and owned `VertexLayout`s.

- WGSL comes from a `ShaderSource`, which composes a module by name with its imports resolved.
- `warm` makes a level's pipelines before the frames that use them.
- `invalidate_module` drops a module hot reload changed and every pipeline made from it; `invalidate_all` drops
  everything. Each bumps `get_generation`, so pipeline holders know to ask again.

**Holding pipelines.**

- A pass holds what it draws with in a `PipelineSet<T>` (one pipeline, an array, or a struct naming each), read through
  `Deref`, with the generation it was made in.
- `refresh` remakes it once the cache has moved past that generation and returns any refusal, keeping the last good
  set and not retrying until the cache changes again.
- `PipelineSet::ask` makes a pipeline only once a pass actually draws.
- A pass drawing into targets of several formats holds a `PipelineMap<K, T>`: one set per key, made on first use
  (`prepare`) and remade together.

**Errors.** Every creation runs under a wgpu validation scope, so whatever wgpu refuses comes back as an error naming the
module or pipeline. A shader compile error names lines of the composed text; `ShaderLines` maps each back to its module
and line, using the markers a source writes with `ShaderLines::mark`.

**Trusted modules.** Modules are created with `create_shader_module_trusted`: bounds checks and loop bounding off;
division and ray queries still guarded. This is sound only for the renderer's own WGSL, and every new shader must keep
to it:

- every loop must end;
- storage buffers are bound through descriptors the backends bound-check in hardware;
- a texture binding array is indexed only by slots the renderer itself handed out.

**Driver cache.** `PipelineCache::with_driver_cache(device, data)` uses the driver's own pipeline cache where the
device has `Features::PIPELINE_CACHE`.

- `get_driver_data` returns its contents behind a CRC32 checksum, to save and pass back at the next start.
- Data failing the checksum is logged and ignored; wgpu itself refuses data from another adapter or driver.

## Allocators

### `SpanBuffer`

Holds data that persists and changes: GPU buffers of fixed-size elements from which ranges (`Span`) are taken and given
back, so a scene's contents can be removed as well as added.

- Its lanes (`SpanLane`) are parallel buffers sharing one allocator, so a span addresses the same elements in each.
- `SpanAllocator` is the bookkeeping, with no GPU. Each growth adds a region with its own `offset-allocator` (O(1),
  binned, cannot grow in place), so a held span keeps its offset.
- A new region is at least twice the request, because the allocator rounds a request up to its size bin and a region
  of exactly the request's size may not fit it.
- Growing copies each lane into a buffer at least twice the size and submits the copy at once. It fails past the
  device's `max_buffer_size`. `get_generation` changes, so anything bound to the old buffers is rebuilt.
- Writes are queued, and `flush` uploads them as contiguous runs (`SpanWrites`); a later write to the same bytes wins.
- A freed span keeps its bytes until overwritten, so `clear` it first where a shader walks the whole buffer.
- Every stride is a multiple of four bytes, since copies move whole words.

### `UploadRing`

Holds what one frame needs.

- Values pushed during the frame are packed at offsets aligned for both uniform and storage dynamic offsets
  (`UploadSlice`) and uploaded in one write at `flush`.
- `flush` first replaces the buffer with a larger one if the frame outgrew it (`get_generation` again).
- A value is valid from that `flush` until the next frame's; the queue orders that after the frame that read it.

## Scene stores

A scene's items live in stores, one per kind. `ProxyStore<T>` keeps them as dense records the GPU mirrors element for
element.

- An add appends and returns a `ProxyHandle<T>`.
- A remove moves the last record into the gap, so records stay dense; the moved item keeps its handle.
- Handles are generational: once an item is gone its handle answers nothing, even after the slot is reused.
- Every record added, changed (`get_mut`) or moved is noted. `StoreMirror::sync` uploads those as contiguous runs into a
  GPU buffer of the store's records, growing it by a GPU copy (`get_generation` again).

When items are made before the store that holds them (as a world posts items to a scene), a `ProxyAllocator<T>` hands
out the handles and the store takes each item at its handle with `insert`. A store takes its handles from `add` or from
an allocator, never both.

## Generated files

`GeneratedShaderFile::sync` keeps a checked-in WGSL file written from Rust declarations (`ShaderDeclarations`, pass
parameters' `get_wgsl_bindings`, permutations' `get_wgsl_overrides`), the same way the TypeScript bindings are kept: a
test syncs it, rewrites it under a generated header when stale (line endings aside), and fails once. It passes once the
file is committed.

## Checks

Run `cargo test --locked -p xrf-renderer-core`.

- The graph's compile tests need no GPU.
- The execution, allocator, mirror, pipeline and parameter tests open a wgpu device, falling back to the software
  adapter (WARP under D3D12). On a machine with neither they skip with a message, or fail where `XRF_TEST_REQUIRE_GPU`
  is set, as Windows CI sets it.

Sources: the [frame graph](src/graph/frame_graph.rs) and its [compiler](src/graph/compile/graph_compiler.rs), the
[pipeline cache](src/pipeline/pipeline_cache.rs), the [bind group cache](src/param/bind_group_cache.rs), the
[span buffer](src/alloc/span_buffer.rs) and the [proxy store](src/store/proxy_store.rs).
