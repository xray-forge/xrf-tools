// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(2) @binding(0) var<storage, read> clusters: array<Cluster>;
@group(2) @binding(1) var<storage, read> slots: array<Slot>;
@group(2) @binding(2) var<storage, read> places: array<Place>;
@group(2) @binding(3) var<storage, read> surfaces: array<Surface>;
@group(2) @binding(4) var<storage, read> indices: array<u32>;
@group(2) @binding(5) var<storage, read> lists: array<vec2<u32>>;
@group(2) @binding(6) var<storage, read> words: array<u32>;
@group(2) @binding(7) var<uniform> wind: Wind;
@group(2) @binding(8) var<storage, read> skins: array<u32>;
@group(2) @binding(9) var<storage, read> bones: array<vec4<f32>>;
