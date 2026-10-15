#import "common/fullscreen"
#import "generated/frame/shadow_tile_restore"

// Restores the depth under a pass's viewport, a tile of the light shadow atlas, from the same tile of the static atlas:
// what stands still, which the swaying casters are then drawn over.

@fragment
fn fs_restore(@builtin(position) position: vec4<f32>) -> @builtin(frag_depth) f32 {
  return textureLoad(static_atlas, vec2<i32>(position.xy), 0);
}
