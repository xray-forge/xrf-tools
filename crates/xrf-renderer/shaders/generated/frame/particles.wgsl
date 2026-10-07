// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var<storage, read> vertices: array<ParticleVertex>;
@group(1) @binding(1) var<storage, read> surfaces: array<ParticleSurface>;
@group(1) @binding(2) var<uniform> lighting: Lighting;
@group(1) @binding(3) var depth_target: texture_depth_2d;
@group(1) @binding(4) var clamped_sampler: sampler;
