// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var<storage, read> clusters: array<Cluster>;
@group(1) @binding(1) var<storage, read> spheres: array<vec4<f32>>;
@group(1) @binding(2) var<storage, read> slots: array<Slot>;
@group(1) @binding(3) var<storage, read> places: array<Place>;
@group(1) @binding(4) var<storage, read> rows: array<Row>;
@group(1) @binding(5) var<storage, read> regions: array<Region>;
@group(1) @binding(6) var<storage, read_write> lists: array<vec2<u32>>;
@group(1) @binding(7) var<storage, read_write> args: array<atomic<u32>>;
@group(1) @binding(8) var<uniform> params: CullParams;
@group(1) @binding(9) var<storage, read_write> candidates: array<vec2<u32>>;
@group(1) @binding(10) var<storage, read_write> late: array<atomic<u32>>;
@group(1) @binding(11) var pyramid: texture_2d<f32>;
@group(1) @binding(12) var<uniform> occlusion: Occlusion;
@group(1) @binding(13) var<storage, read> impostors: array<Impostor>;
@group(1) @binding(14) var<storage, read_write> terms: array<vec4<u32>>;
@group(1) @binding(15) var<storage, read_write> impostor_list: array<u32>;
