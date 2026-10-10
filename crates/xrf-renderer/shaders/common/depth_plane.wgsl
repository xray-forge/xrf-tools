// The plane the frame's depth lies in about a pixel, from the points its neighbours show: the surface's own, where the
// G-buffer's normal is bumped by tens of degrees. A point carries one in `w` where something was drawn there.

// Of the steps from a point to its two neighbours along an axis, the one along its own surface: the nearer in depth, a
// drawn one before none.
fn surface_step(center: vec3<f32>, before: vec4<f32>, after: vec4<f32>) -> vec3<f32> {
  let behind: vec3<f32> = center - before.xyz;
  let ahead: vec3<f32> = after.xyz - center;
  let is_ahead: bool = after.w > 0.5 && (before.w < 0.5 || abs(ahead.z) < abs(behind.z));

  return select(behind, ahead, is_ahead);
}

// The normal of the plane through a view space point and its neighbours across and down the screen (`left`, `right`,
// `up`, `down`), each axis's step along its own surface so an edge does not lean it; facing the eye.
fn surface_normal(center: vec3<f32>, left: vec4<f32>, right: vec4<f32>, up: vec4<f32>, down: vec4<f32>) -> vec3<f32> {
  let crossed: vec3<f32> = normalize(cross(surface_step(center, up, down), surface_step(center, left, right)));

  return select(crossed, -crossed, dot(crossed, center) > 0.0);
}
