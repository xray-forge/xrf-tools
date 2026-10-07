// What the present pass and the overlays drawn over it share; `Present` is declared in Rust.

// The drawn texel under a point of a viewport `size` pixels across whose scene is drawn `drawn` texels across.
fn to_drawn_texel(pixel: vec2<f32>, drawn: vec2<f32>, size: vec2<f32>) -> vec2<i32> {
  return vec2<i32>(clamp(floor((pixel + 0.5) * drawn / size), vec2<f32>(0.0), drawn - 1.0));
}
