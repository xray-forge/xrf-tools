// Two directions across a unit axis, at right angles to it and to each other, which samples spread about it by.
struct AxisFrame {
  across: vec3<f32>,
  along: vec3<f32>,
}

// The frame about a unit `axis`, its first side across the world's up unless the axis stands near it.
fn axis_frame(axis: vec3<f32>) -> AxisFrame {
  let side: vec3<f32> = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(axis.y) > 0.9);
  let across: vec3<f32> = normalize(cross(axis, side));

  return AxisFrame(across, cross(axis, across));
}
