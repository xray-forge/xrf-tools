// The sRGB display's transfer, which the engine's colours were authored on and its albedo holds as it is: the 2.2
// power of the standard's reference display (IEC 61966-2-1), not the encoding's piecewise curve. The game's colours
// were judged multiplied as display values, a dark texture's channel by a dim light's; the piecewise curve's linear toe
// would drop such a product's weak channel several times under what the engine showed (a low sun's shade warmer and
// more saturated than the game's), where the power gives each product back as the engine multiplied it.

// Encoded values back to linear light.
fn srgb_decoded(color: vec3<f32>) -> vec3<f32> {
  return pow(max(color, vec3<f32>(0.0)), vec3<f32>(2.2));
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

// Linear light encoded by the sRGB display's transfer, as a display takes it.
fn srgb_encoded(color: vec3<f32>) -> vec3<f32> {
  return pow(max(color, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}
