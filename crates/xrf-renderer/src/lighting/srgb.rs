use glam::Vec3;

/// Linear light encoded by the sRGB transfer, as a colour texture holds it; `common/srgb.wgsl` decodes it back.
pub fn srgb_encoded(linear: f32) -> f32 {
  if linear <= 0.003_130_8 {
    linear * 12.92
  } else {
    1.055 * linear.powf(1.0 / 2.4) - 0.055
  }
}

/// A value encoded by the sRGB transfer back to linear light, as `common/srgb.wgsl`'s `srgb_decoded`; past one, as an
/// engine light brighter than white is, the transfer's curve continues.
pub fn srgb_decoded(encoded: f32) -> f32 {
  if encoded <= 0.040_45 {
    encoded / 12.92
  } else {
    ((encoded + 0.055) / 1.055).powf(2.4)
  }
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

  let luminance = |colour: Vec3| colour.dot(Vec3::new(0.2126, 0.7152, 0.0722));
  let hue: Vec3 = authored_colour(colour.max(Vec3::ZERO) / peak, true);

  hue * (luminance(colour) / luminance(hue).max(1e-12))
}
