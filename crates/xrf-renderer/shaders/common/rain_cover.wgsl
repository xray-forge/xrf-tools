// What stands over the rain around the camera, as `scene/level/rain_cover.rs` draws it straight down: for each column,
// how high the first thing a drop lands on is.

// Texels the cover is across, and metres it reaches down from the height it is seen from.
const RAIN_COVER_RESOLUTION: f32 = 1024.0;
const RAIN_COVER_DEPTH: f32 = 120.0;

// The height of the first thing over a column a drop lands on, in renderer space: read from the cover's depth,
// reversed, straight down from the height it is seen from; nothing at all outside it. `window` is the cover's centre
// in `x` and `z`, its half width, and the height it is seen from.
fn rain_cover_height(cover: texture_depth_2d, window: vec4<f32>, column: vec3<f32>) -> f32 {
  let uv: vec2<f32> = vec2<f32>(column.x - window.x, column.z - window.y) / (window.z * 2.0) + 0.5;

  if (any(uv < vec2<f32>(0.0)) || any(uv >= vec2<f32>(1.0))) {
    return -1e9;
  }

  let texel: vec2<i32> = vec2<i32>(clamp(uv * RAIN_COVER_RESOLUTION, vec2<f32>(0.0), vec2<f32>(RAIN_COVER_RESOLUTION - 1.0)));

  return window.w - (1.0 - textureLoad(cover, texel, 0)) * RAIN_COVER_DEPTH;
}
