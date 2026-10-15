#import "common/fullscreen"
#import "generated/frame/overhead_shift"

// Shifts an overhead map moved a whole step: each texel takes the depth its old self holds where the map covered it
// before, and is left to the far plane, its pass's clear, where it did not; the strips it did not cover are drawn over
// after.

@fragment
fn fs_shift(@builtin(position) position: vec4<f32>) -> @builtin(frag_depth) f32 {
  let source: vec2<i32> = vec2<i32>(position.xy) + vec2<i32>(shift.offset.xy);
  let size: i32 = i32(shift.offset.z);

  if (any(source < vec2<i32>(0)) || any(source >= vec2<i32>(size))) {
    discard;
  }

  return textureLoad(previous, source, 0);
}
