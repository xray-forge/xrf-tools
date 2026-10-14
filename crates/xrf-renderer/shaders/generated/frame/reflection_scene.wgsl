// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var<uniform> lighting: Lighting;
@group(1) @binding(1) var material_lut: texture_3d<f32>;
@group(1) @binding(2) var lut_sampler: sampler;
@group(1) @binding(3) var<storage, read> exposure: Exposure;
@group(1) @binding(4) var<storage, read> trace_requests: array<TraceRequest>;
@group(1) @binding(5) var<storage, read_write> trace_request_head: array<u32>;
@group(1) @binding(6) var traced_out: texture_storage_2d<rgba16float, write>;
@group(1) @binding(7) var ends_out: texture_storage_2d<r32float, write>;
