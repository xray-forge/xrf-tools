// What every pass drawing a viewport knows of its camera and the frame's switches.
struct Camera {
  view_projection: mat4x4<f32>,
  inverse_view_projection: mat4x4<f32>,
  view: mat4x4<f32>,
  projection: mat4x4<f32>,
  inverse_projection: mat4x4<f32>,
  // xyz: the eye in renderer space.
  position: vec4<f32>,
  // xy: the viewport's size in device pixels; zw: its top left corner in the window.
  viewport: vec4<f32>,
  // The frustum's six planes in renderer space, pointing inward.
  planes: array<vec4<f32>, 6>,
  // x: textured, y: bumped, z: the baked hemisphere's strength, w: how far the water distorts what is behind it.
  switches: vec4<f32>,
  // x: one where every static surface draws as its triangles' edges; y: times a uv checker repeats in place of every
  // surface's textures, zero for none; z: one where surfaces draw solid, their alpha ignored.
  modes: vec4<f32>,
  // World to clip without the jitter, this frame and the last, which a surface's motion is measured by.
  motion_current: mat4x4<f32>,
  motion_previous: mat4x4<f32>,
  // What shows where nothing was drawn and neither the sky nor the fog is; `w` one where it is set.
  backdrop: vec4<f32>,
  // What a surface naming no base texture is drawn; `w` one where it is set, white where it is not.
  plain: vec4<f32>,
  // The backdrop's second colour, and in `w` the side of a square in render pixels; zero for a plain backdrop.
  backdrop_squares: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;

// How far a surface's point moved on the screen since the last frame, from where it stood then to where it stands
// now, each through its own frame's unjittered camera: in texture coordinates, now less then, `y` down.
fn camera_motion(world: vec3<f32>, previous_world: vec3<f32>) -> vec2<f32> {
  let current: vec4<f32> = camera.motion_current * vec4<f32>(world, 1.0);
  let previous: vec4<f32> = camera.motion_previous * vec4<f32>(previous_world, 1.0);

  return (current.xy / current.w - previous.xy / previous.w) * vec2<f32>(0.5, -0.5);
}

// The point of the view frustum under a viewport position at a depth, reversed: one is the near plane.
fn camera_unproject(ndc: vec2<f32>, depth: f32) -> vec3<f32> {
  let point: vec4<f32> = camera.inverse_view_projection * vec4<f32>(ndc, depth, 1.0);

  return point.xyz / point.w;
}

// The view space point under a pixel of the viewport's own targets at a depth, reversed: one is the near plane.
fn camera_view_position(pixel: vec2<f32>, depth: f32) -> vec3<f32> {
  let size: vec2<f32> = camera.viewport.xy;
  let ndc: vec2<f32> = vec2<f32>(pixel.x / size.x * 2.0 - 1.0, 1.0 - pixel.y / size.y * 2.0);
  let point: vec4<f32> = camera.inverse_projection * vec4<f32>(ndc, depth, 1.0);

  return point.xyz / point.w;
}
