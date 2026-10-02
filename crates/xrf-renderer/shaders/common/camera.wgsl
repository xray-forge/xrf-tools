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
};

@group(0) @binding(0) var<uniform> camera: Camera;

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
