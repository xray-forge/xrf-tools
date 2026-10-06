// Reduces a viewport's depth to the nearest depth under each texel at the size the reflections' rays are traced at,
// which they are marched over: with reversed depth the nearest is the largest. Each texel stands for the frame's pixels
// it covers exactly.

// The depth pyramid's own bindings: the depth read, the reduction written.
#import "generated/frame/pyramid"

// Each texel the nearest of the frame's pixels it stands for.
@compute @workgroup_size(8, 8)
fn reduce_nearest_depth(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<u32> = textureDimensions(target_level);

  if (id.x >= size.x || id.y >= size.y) {
    return;
  }

  let depth_size: vec2<u32> = textureDimensions(source_depth);
  let ratio: vec2<u32> = (depth_size + size - 1u) / size;
  let start: vec2<u32> = id.xy * ratio;
  let end: vec2<u32> = min(start + ratio, depth_size);
  var nearest: f32 = 0.0;

  for (var y: u32 = start.y; y < end.y; y++) {
    for (var x: u32 = start.x; x < end.x; x++) {
      nearest = max(nearest, textureLoad(source_depth, vec2<u32>(x, y), 0));
    }
  }

  textureStore(target_level, id.xy, vec4<f32>(nearest, 0.0, 0.0, 0.0));
}
