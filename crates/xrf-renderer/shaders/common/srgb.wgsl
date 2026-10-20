// The sRGB transfer, which the engine's colour textures are encoded by and its albedo holds as it is.

// Encoded values back to linear light.
fn srgb_decoded(color: vec3<f32>) -> vec3<f32> {
  return select(pow((color + 0.055) / 1.055, vec3<f32>(2.4)), color / 12.92, color <= vec3<f32>(0.04045));
}

// Linear light encoded by the sRGB transfer, as a display takes it.
fn srgb_encoded(color: vec3<f32>) -> vec3<f32> {
  let clamped: vec3<f32> = max(color, vec3<f32>(0.0));

  return select(1.055 * pow(clamped, vec3<f32>(1.0 / 2.4)) - 0.055, clamped * 12.92, clamped <= vec3<f32>(0.0031308));
}
