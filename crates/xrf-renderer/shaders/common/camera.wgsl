// What every pass drawing a viewport knows of its camera.
struct Camera {
  view_projection: mat4x4<f32>,
  inverse_view_projection: mat4x4<f32>,
  // xyz: the eye in renderer space.
  position: vec4<f32>,
  // xy: the viewport's size in device pixels.
  viewport: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;

// The point of the view frustum under a viewport position at a depth, reversed: one is the near plane.
fn camera_unproject(ndc: vec2<f32>, depth: f32) -> vec3<f32> {
  let point: vec4<f32> = camera.inverse_view_projection * vec4<f32>(ndc, depth, 1.0);

  return point.xyz / point.w;
}
