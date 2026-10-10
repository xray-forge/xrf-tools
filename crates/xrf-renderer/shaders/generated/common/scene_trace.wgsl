// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

enable wgpu_ray_query;
@group(3) @binding(0) var trace_scene: acceleration_structure;
@group(3) @binding(1) var<storage, read> trace_instances: array<TraceInstance>;
@group(3) @binding(2) var<storage, read> trace_records: array<TraceRecord>;
@group(3) @binding(3) var<storage, read> trace_places: array<Place>;
@group(3) @binding(4) var<storage, read> trace_baked_words: array<u32>;
@group(3) @binding(5) var<storage, read> trace_tree_words: array<u32>;
@group(3) @binding(6) var<storage, read> trace_model_words: array<u32>;
@group(3) @binding(7) var<storage, read> trace_indices: array<u32>;
@group(3) @binding(8) var<storage, read> trace_surfaces: array<Surface>;
@group(3) @binding(9) var<storage, read> trace_lights: array<LightRecord>;
@group(3) @binding(10) var<uniform> trace_light_view: Lights;
@group(3) @binding(11) var trace_shadow_atlas: texture_depth_2d;
@group(3) @binding(12) var trace_shadow_sampler: sampler_comparison;
@group(3) @binding(13) var trace_sun_maps: texture_depth_2d_array;
@group(3) @binding(14) var<uniform> trace_sun_shadows: Shadows;
@group(3) @binding(15) var<storage, read> trace_grass_grid: array<u32>;
@group(3) @binding(16) var<storage, read> trace_grass_slots: array<u32>;
@group(3) @binding(17) var<storage, read> trace_grass_models: array<GrassModel>;
@group(3) @binding(18) var<storage, read> trace_grass_colours: array<vec4<f32>>;
@group(3) @binding(19) var<uniform> trace_grass: TraceGrass;
@group(3) @binding(20) var<storage, read> trace_grass_cards: array<u32>;
