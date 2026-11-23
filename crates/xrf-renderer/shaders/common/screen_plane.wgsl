#import "common/camera"
#import "common/depth_plane"

// The plane a drawn pixel's depth lies in, read from the frame's depth about it: the surface's own, where the
// G-buffer's normal is bumped by tens of degrees. The depth is passed in, so each pass binds it where it likes.

// The view space point a neighbouring texel shows, one in `w` where something is drawn there.
fn screen_neighbour(depth_target: texture_depth_2d, texel: vec2<i32>, pixel: vec2<f32>) -> vec4<f32> {
  let depth: f32 = textureLoad(depth_target, texel, 0);

  return vec4<f32>(camera_view_position(pixel, max(depth, 1e-7)), select(0.0, 1.0, depth > 0.0));
}

// The normal of the plane a pixel's depth lies in, in view space and facing the eye: across and down, each to whichever
// neighbour lies nearer along the view, so an edge does not bend it.
fn screen_plane_normal(
  depth_target: texture_depth_2d,
  texel: vec2<i32>,
  pixel: vec2<f32>,
  position: vec3<f32>
) -> vec3<f32> {
  let last: vec2<i32> = vec2<i32>(camera.viewport.xy) - 1;
  let x: vec2<i32> = vec2<i32>(1, 0);
  let y: vec2<i32> = vec2<i32>(0, 1);

  return surface_normal(
    position,
    screen_neighbour(depth_target, max(texel - x, vec2<i32>(0)), pixel - vec2<f32>(x)),
    screen_neighbour(depth_target, min(texel + x, last), pixel + vec2<f32>(x)),
    screen_neighbour(depth_target, max(texel - y, vec2<i32>(0)), pixel - vec2<f32>(y)),
    screen_neighbour(depth_target, min(texel + y, last), pixel + vec2<f32>(y)),
  );
}
