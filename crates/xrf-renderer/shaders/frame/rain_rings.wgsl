// Reduces the rain's rings (`scene/level/rain_rings.rs`) from their first level into the four after it in one
// dispatch: each workgroup averages a block of 16 texels a side down to one, through workgroup memory, so no level
// waits on another pass.

#import "generated/frame/rain_rings"

// Each level's texels the workgroup holds, the second's 8 a side first.
var<workgroup> reduced: array<vec4<f32>, 64>;

@compute @workgroup_size(8, 8)
fn cs_reduce_rings(@builtin(local_invocation_id) local: vec3<u32>, @builtin(workgroup_id) group: vec3<u32>) {
  let texel: vec2<u32> = group.xy * 8u + local.xy;
  let first: vec2<u32> = texel * 2u;
  let second: vec4<f32> = 0.25 * (textureLoad(rings_first, first, 0) +
    textureLoad(rings_first, first + vec2<u32>(1u, 0u), 0) + textureLoad(rings_first, first + vec2<u32>(0u, 1u), 0) +
    textureLoad(rings_first, first + vec2<u32>(1u, 1u), 0));

  textureStore(rings_second, texel, second);
  reduced[local.y * 8u + local.x] = second;
  workgroupBarrier();

  // Each level from the four texels under it in the last, by a quarter of the invocations before; read in full before
  // any is written over.
  let is_third: bool = all(local.xy < vec2<u32>(4u));
  let is_fourth: bool = all(local.xy < vec2<u32>(2u));
  let under: u32 = local.y * 16u + local.x * 2u;
  let held: u32 = local.y * 8u + local.x;
  var third: vec4<f32> = vec4<f32>(0.0);

  if (is_third) {
    third = 0.25 * (reduced[under] + reduced[under + 1u] + reduced[under + 8u] + reduced[under + 9u]);
    textureStore(rings_third, group.xy * 4u + local.xy, third);
  }

  workgroupBarrier();

  if (is_third) {
    reduced[held] = third;
  }

  workgroupBarrier();

  var fourth: vec4<f32> = vec4<f32>(0.0);

  if (is_fourth) {
    fourth = 0.25 * (reduced[under] + reduced[under + 1u] + reduced[under + 8u] + reduced[under + 9u]);
    textureStore(rings_fourth, group.xy * 2u + local.xy, fourth);
  }

  workgroupBarrier();

  if (is_fourth) {
    reduced[held] = fourth;
  }

  workgroupBarrier();

  if (all(local.xy == vec2<u32>(0u))) {
    textureStore(rings_fifth, group.xy, 0.25 * (reduced[0] + reduced[1] + reduced[8] + reduced[9]));
  }
}
