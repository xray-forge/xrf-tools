// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var depth_target: texture_depth_2d;
@group(1) @binding(1) var<storage, read> cells: array<u32>;
@group(1) @binding(2) var<storage, read> nodes: array<u32>;
@group(1) @binding(3) var<uniform> ai_map: AiMap;
