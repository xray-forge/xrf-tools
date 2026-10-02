// What the present pass and the overlays drawn over it read of the viewport, as `pass/present_uniform.rs` writes it.

struct Present {
  // Which picture: the scene at zero, else a target, in `RenderDebugView`'s order.
  view: u32,
  // Whether the screen's occlusion was searched this frame.
  is_occluded: u32,
  // Whether the frame shown is the upscaled one, at the viewport's size, rather than the scene as drawn.
  is_upscaled: u32,
  pad: u32,
  // The viewport's top left corner in the window and its size, in pixels; the scene is drawn at `camera.viewport.xy`.
  origin: vec2<f32>,
  size: vec2<f32>,
  // `img_corrections`: x exposure, y gamma, z saturation; then the grading colour.
  corrections: vec4<f32>,
  grading: vec4<f32>,
};

// The drawn texel under a point of a viewport `size` pixels across whose scene is drawn `drawn` texels across.
fn to_drawn_texel(pixel: vec2<f32>, drawn: vec2<f32>, size: vec2<f32>) -> vec2<i32> {
  return vec2<i32>(clamp(floor((pixel + 0.5) * drawn / size), vec2<f32>(0.0), drawn - 1.0));
}
