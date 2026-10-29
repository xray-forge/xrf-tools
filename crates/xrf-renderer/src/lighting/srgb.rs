use glam::Vec3;

use crate::lighting::luminance::luminance;

/// The power of the sRGB display's transfer: the standard's reference display, which `common/srgb.wgsl` explains.
pub const SRGB_DISPLAY_POWER: f32 = 2.2;

/// Linear light encoded by the sRGB display's transfer, as a colour texture holds it; `common/srgb.wgsl` decodes it
/// back.
pub fn srgb_encoded(linear: f32) -> f32 {
  linear.max(0.0).powf(1.0 / SRGB_DISPLAY_POWER)
}

/// A value encoded by the sRGB display's transfer back to linear light, as `common/srgb.wgsl`'s `srgb_decoded`; past
/// one, as an engine light brighter than white is, the power continues.
pub fn srgb_decoded(encoded: f32) -> f32 {
  encoded.max(0.0).powf(SRGB_DISPLAY_POWER)
}

/// A colour authored as a display value (a picture's, a lamp's) as the frame lights with it, as
/// `common/scene_light.wgsl`'s `authored_colour`: decoded where the light is physical, as it stands where it is the
/// engine's.
pub fn authored_colour(colour: Vec3, is_physical: bool) -> Vec3 {
  if is_physical {
    Vec3::new(srgb_decoded(colour.x), srgb_decoded(colour.y), srgb_decoded(colour.z))
  } else {
    colour
  }
}

/// One of the weather's lights authored as a display value (its sun, ambient, hemisphere colour) as the frame lights
/// with it, as `common/srgb.wgsl`'s `srgb_light`: where the light is physical its hue decoded and its luminance kept,
/// since the game balanced the weather's lights against each other by their display values; as it stands where it is
/// the engine's.
pub fn authored_light(colour: Vec3, is_physical: bool) -> Vec3 {
  let peak: f32 = colour.max_element();

  if !is_physical || peak <= 0.0 {
    return if is_physical { Vec3::ZERO } else { colour };
  }

  let hue: Vec3 = authored_colour(colour.max(Vec3::ZERO) / peak, true);

  hue * (luminance(colour) / luminance(hue).max(1e-12))
}

/// A light authored as a display value as additive linear light over the lights `under` it, in the engine's units: what
/// the display's decode of their sum adds to its decode of `under` alone, per channel. Lights anchored over each other
/// in turn sum, where all of them shine, to the decode of the engine's sum, and add linearly where only some do.
pub fn anchored_light(light: Vec3, under: Vec3) -> Vec3 {
  let under: Vec3 = under.max(Vec3::ZERO);

  (authored_colour(light.max(Vec3::ZERO) + under, true) - authored_colour(under, true)).max(Vec3::ZERO)
}
