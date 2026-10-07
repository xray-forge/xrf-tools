// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

enable wgpu_ray_query;
@group(3) @binding(0) var trace_scene: acceleration_structure;
@group(3) @binding(1) var<storage, read> trace_records: array<TraceRecord>;
@group(3) @binding(2) var<storage, read> trace_words: array<u32>;
@group(3) @binding(3) var<storage, read> trace_indices: array<u32>;
@group(3) @binding(4) var<storage, read> trace_surfaces: array<Surface>;
