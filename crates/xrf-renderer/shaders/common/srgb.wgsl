// The sRGB transfer, which the engine's colour textures are encoded by and its albedo holds as it is.

// Encoded values back to linear light.
fn srgb_decoded(color: vec3<f32>) -> vec3<f32> {
  return select(pow((color + 0.055) / 1.055, vec3<f32>(2.4)), color / 12.92, color <= vec3<f32>(0.04045));
}
