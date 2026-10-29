// The sRGB display's transfer, which the engine's colours were authored on and its albedo holds as it is: the 2.2
// power of the standard's reference display (IEC 61966-2-1), not the encoding's piecewise curve. The game's colours
// were judged multiplied as display values, a dark texture's channel by a dim light's; the piecewise curve's linear toe
// would drop such a product's weak channel several times under what the engine showed (a low sun's shade warmer and
// more saturated than the game's), where the power gives each product back as the engine multiplied it.

// The reference display's power.
const SRGB_DISPLAY_POWER: f32 = 2.2;

// Encoded values back to linear light.
fn srgb_decoded(color: vec3<f32>) -> vec3<f32> {
  return pow(max(color, vec3<f32>(0.0)), vec3<f32>(SRGB_DISPLAY_POWER));
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

// A light `srgb_light` made back to the display value it was authored as: its hue encoded again, its luminance kept.
fn srgb_light_authored(light: vec3<f32>) -> vec3<f32> {
  let luminance_weights: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
  let peak: f32 = max(max(light.r, light.g), light.b);

  if (peak <= 0.0) {
    return vec3<f32>(0.0);
  }

  let hue: vec3<f32> = pow(max(light, vec3<f32>(0.0)) / peak, vec3<f32>(1.0 / SRGB_DISPLAY_POWER));

  return hue * (dot(light, luminance_weights) / max(dot(hue, luminance_weights), 1e-12));
}

// Linear light encoded by the sRGB display's transfer, as a display takes it.
fn srgb_encoded(color: vec3<f32>) -> vec3<f32> {
  return pow(max(color, vec3<f32>(0.0)), vec3<f32>(1.0 / SRGB_DISPLAY_POWER));
}

// A light authored as a display value as additive linear light over the lights `under` it, both display values: what
// the display's decode of their sum adds to its decode of `under` alone, per channel, as `anchored_light` on the CPU.
fn anchored_light(light: vec3<f32>, under: vec3<f32>) -> vec3<f32> {
  let below: vec3<f32> = max(under, vec3<f32>(0.0));

  return max(srgb_decoded(max(light, vec3<f32>(0.0)) + below) - srgb_decoded(below), vec3<f32>(0.0));
}
