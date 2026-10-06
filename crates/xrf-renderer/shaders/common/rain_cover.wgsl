// Overhead maps of the level around the camera, as `scene/level/overhead_map.rs` draws them straight down: for each
// column, how high the first thing over it stands. The rain's cover is one; the level's surface without its trees is
// another.

// The cover's texels across, and metres it reaches down from the height it is seen from.
const RAIN_COVER_RESOLUTION: f32 = 1024.0;
const RAIN_COVER_DEPTH: f32 = 120.0;

// The height of the first thing over a column in an overhead map, in renderer space: read from its depth, reversed,
// straight down from the height it is seen from; nothing at all outside it. `window` is its centre in `x` and `z`, its
// half width, and the height it is seen from; `resolution` its texels across and `reach` the metres it reaches down.
fn overhead_height(map: texture_depth_2d, window: vec4<f32>, resolution: f32, reach: f32, column: vec3<f32>) -> f32 {
  let uv: vec2<f32> = vec2<f32>(column.x - window.x, column.z - window.y) / (window.z * 2.0) + 0.5;

  if (any(uv < vec2<f32>(0.0)) || any(uv >= vec2<f32>(1.0))) {
    return -1e9;
  }

  let texel: vec2<i32> = vec2<i32>(clamp(uv * resolution, vec2<f32>(0.0), vec2<f32>(resolution - 1.0)));

  return window.w - (1.0 - textureLoad(map, texel, 0)) * reach;
}

// The height of the first thing over a column a drop lands on.
fn rain_cover_height(cover: texture_depth_2d, window: vec4<f32>, column: vec3<f32>) -> f32 {
  return overhead_height(cover, window, RAIN_COVER_RESOLUTION, RAIN_COVER_DEPTH, column);
}
