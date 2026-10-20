#import "common/camera"
#import "common/sky_box"
#import "generated/structs"

// The distance along the view the reflection's history keeps for a pixel whose ray met the sky; none where no water
// drew.
const SKY_DISTANCE: f32 = 60000.0;

// How far along the view the G-buffer's surface lies at a place on the screen; nothing where only the sky is. The
// perspective's depth terms alone give it, as its `w` is the distance: `depth * d = P[3][2] - P[2][2] * d`.
fn scene_distance(uv: vec2<f32>, depth_target: texture_depth_2d) -> f32 {
  let size: vec2<f32> = camera.viewport.xy;
  let pixel: vec2<f32> = clamp(floor(uv * size), vec2<f32>(0.0), size - 1.0);
  let stored: f32 = textureLoad(depth_target, vec2<i32>(pixel), 0);

  return select(camera_view_distance(stored), 0.0, stored <= 0.0);
}
