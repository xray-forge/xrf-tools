// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var<storage, read> records: array<LightRecord>;
@group(0) @binding(1) var<storage, read_write> counts: array<atomic<u32>>;
@group(0) @binding(2) var<storage, read_write> items: array<u32>;
@group(0) @binding(3) var<uniform> lights: Lights;
