// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(2) @binding(0) var<storage, read> sorted: array<vec4<f32>>;
@group(2) @binding(1) var<storage, read> models: array<GrassModel>;
@group(2) @binding(2) var<uniform> wind: GrassWind;
