// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(2) @binding(0) var<storage, read> positions: array<f32>;
@group(2) @binding(1) var<storage, read> uvs: array<f32>;
@group(2) @binding(2) var<storage, read> indices: array<u32>;
@group(2) @binding(3) var<uniform> wind: GrassWind;
@group(2) @binding(4) var<storage, read_write> card_vertices: array<u32>;
@group(2) @binding(5) var<storage, read_write> card_triangles: array<u32>;
@group(2) @binding(6) var<storage, read_write> card_count: atomic<u32>;
@group(2) @binding(7) var<storage, read_write> card_draw: array<u32, 4>;
