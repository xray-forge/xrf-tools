// Reduces a viewport's depth to a pyramid of the farthest depth under each texel, which occlusion tests read, or of the
// nearest, which reflected rays skip empty space by: with reversed depth the farthest is the smallest, the nearest the
// largest.

#import "generated/frame/pyramid"

// The depth's first level: a power of two at most the depth's size, so each texel covers up to three depth texels a
// side and is reduced over all of them.
@compute @workgroup_size(8, 8)
fn reduce_depth(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<u32> = textureDimensions(target_level);

  if (id.x >= size.x || id.y >= size.y) {
    return;
  }

  let depth_size: vec2<u32> = textureDimensions(source_depth);
  let start: vec2<u32> = id.xy * depth_size / size;
  let end: vec2<u32> = min(((id.xy + 1u) * depth_size + size - 1u) / size, depth_size);
  var farthest: f32 = 1.0;

  for (var y: u32 = start.y; y < end.y; y++) {
    for (var x: u32 = start.x; x < end.x; x++) {
      farthest = min(farthest, textureLoad(source_depth, vec2<u32>(x, y), 0));
    }
  }

  textureStore(target_level, id.xy, vec4<f32>(farthest, 0.0, 0.0, 0.0));
}

@compute @workgroup_size(8, 8)
fn reduce_level(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<u32> = textureDimensions(target_level);

  if (id.x >= size.x || id.y >= size.y) {
    return;
  }

  let at: vec2<u32> = id.xy * 2u;
  let last: vec2<u32> = textureDimensions(source_level) - 1u;
  let farthest: f32 = min(
    min(textureLoad(source_level, at, 0).x, textureLoad(source_level, min(at + vec2<u32>(1u, 0u), last), 0).x),
    min(
      textureLoad(source_level, min(at + vec2<u32>(0u, 1u), last), 0).x,
      textureLoad(source_level, min(at + 1u, last), 0).x
    )
  );

  textureStore(target_level, id.xy, vec4<f32>(farthest, 0.0, 0.0, 0.0));
}

// The nearest's first level: the depth itself, texel for texel.
@compute @workgroup_size(8, 8)
fn copy_nearest(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<u32> = textureDimensions(target_level);

  if (id.x >= size.x || id.y >= size.y) {
    return;
  }

  textureStore(target_level, id.xy, vec4<f32>(textureLoad(source_depth, id.xy, 0), 0.0, 0.0, 0.0));
}

// A level of the nearest's into the next, half its size rounded down: each texel the nearest of the two by two under it,
// and of the third row or column where the level before is odd and it is the last.
@compute @workgroup_size(8, 8)
fn reduce_nearest(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<u32> = textureDimensions(target_level);

  if (id.x >= size.x || id.y >= size.y) {
    return;
  }

  let source: vec2<u32> = textureDimensions(source_level);
  let at: vec2<u32> = id.xy * 2u;
  let is_odd: vec2<bool> = (source & vec2<u32>(1u)) == vec2<u32>(1u);
  let is_last: vec2<bool> = id.xy == size - 1u;
  let extra: vec2<u32> = select(vec2<u32>(0u), vec2<u32>(1u), is_odd & is_last);
  var nearest: f32 = 0.0;

  for (var y: u32 = 0u; y <= 1u + extra.y; y++) {
    for (var x: u32 = 0u; x <= 1u + extra.x; x++) {
      nearest = max(nearest, textureLoad(source_level, min(at + vec2<u32>(x, y), source - 1u), 0).x);
    }
  }

  textureStore(target_level, id.xy, vec4<f32>(nearest, 0.0, 0.0, 0.0));
}
