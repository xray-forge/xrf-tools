// Reduces a viewport's depth to a pyramid of the farthest depth under each texel, which occlusion tests read: with
// reversed depth the farthest is the smallest.

@group(0) @binding(0) var source_depth: texture_depth_2d;
@group(0) @binding(1) var source_level: texture_2d<f32>;
@group(0) @binding(2) var target_level: texture_storage_2d<r32float, write>;

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
