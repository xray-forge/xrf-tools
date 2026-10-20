/// Linear light encoded by the sRGB transfer, as a colour texture holds it; `common/srgb.wgsl` decodes it back.
pub fn srgb_encoded(linear: f32) -> f32 {
  if linear <= 0.003_130_8 {
    linear * 12.92
  } else {
    1.055 * linear.powf(1.0 / 2.4) - 0.055
  }
}
