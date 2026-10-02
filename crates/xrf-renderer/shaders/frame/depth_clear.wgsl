// Clears the depth under a pass's viewport to the far plane, reversed zero: a tile of an atlas whose other tiles stay.

@vertex
fn vs_clear(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let uv: vec2<f32> = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));

  return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}
