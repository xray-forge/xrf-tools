// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var<storage, read> words: array<u32>;
@group(0) @binding(1) var<storage, read> places: array<Place>;
@group(0) @binding(2) var<storage, read> surfaces: array<Surface>;
@group(0) @binding(3) var<uniform> wind: Wind;
@group(0) @binding(4) var<storage, read> sway_slots: array<TraceSwaySlot>;
@group(0) @binding(5) var<storage, read_write> swayed_vertices: array<f32>;
