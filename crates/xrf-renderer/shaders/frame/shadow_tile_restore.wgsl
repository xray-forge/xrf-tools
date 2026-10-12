#import "generated/frame/shadow_tile_restore"

// Restores the depth under a pass's viewport, a tile of the light shadow atlas, from the same tile of the static atlas:
// what stands still, which the swaying casters are then drawn over.

@vertex
fn vs_restore(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let uv: vec2<f32> = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));

  return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_restore(@builtin(position) position: vec4<f32>) -> @builtin(frag_depth) f32 {
  return textureLoad(static_atlas, vec2<i32>(position.xy), 0);
}
