// The sRGB transfer, which the engine's colour textures are encoded by and its albedo holds as it is.

// Encoded values back to linear light.
fn srgb_decoded(color: vec3<f32>) -> vec3<f32> {
  return select(pow((color + 0.055) / 1.055, vec3<f32>(2.4)), color / 12.92, color <= vec3<f32>(0.04045));
}

// One of the weather's lights the engine authors as a display value (its sun, ambient, hemisphere colour) as linear
// light: its hue decoded, its luminance kept. The game balanced those lights against each other by their display
// values, so their strengths stay linear and only their colour is decoded, as Unity's `lightsUseLinearIntensity` keeps
// a light's intensity; decoding the whole value would raise every ratio between two of them to the transfer's power. It
// scales as its colour does, so a gain can be given before or after.
fn srgb_light(colour: vec3<f32>) -> vec3<f32> {
  let luminance_weights: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
  let peak: f32 = max(max(colour.r, colour.g), colour.b);

  if (peak <= 0.0) {
    return vec3<f32>(0.0);
  }

  let hue: vec3<f32> = srgb_decoded(max(colour, vec3<f32>(0.0)) / peak);

  return hue * (dot(colour, luminance_weights) / max(dot(hue, luminance_weights), 1e-12));
}

// Linear light encoded by the sRGB transfer, as a display takes it.
fn srgb_encoded(color: vec3<f32>) -> vec3<f32> {
  let clamped: vec3<f32> = max(color, vec3<f32>(0.0));

  return select(1.055 * pow(clamped, vec3<f32>(1.0 / 2.4)) - 0.055, clamped * 12.92, clamped <= vec3<f32>(0.0031308));
}
