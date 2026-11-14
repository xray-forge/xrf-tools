use glam::Vec3;

/// The light a point gathers, `base_color_c` (`xrLC_Light/base_color.h`): static colour, sky and sun.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LightColor {
  pub rgb: Vec3,
  pub hemi: f32,
  pub sun: f32,
}

impl LightColor {
  /// `fixed16<16>`'s range: a stored term is a 16-bit fraction of it.
  const FIXED_RANGE: f32 = 16.0;

  /// This light as `base_color` stores it, each term through [`Self::get_fixed`].
  pub fn get_stored(&self) -> Self {
    Self {
      rgb: self.rgb.map(Self::get_fixed),
      hemi: Self::get_fixed(self.hemi),
      sun: Self::get_fixed(self.sun),
    }
  }

  /// Every term times `scale`.
  pub fn get_scaled(&self, scale: f32) -> Self {
    Self {
      rgb: self.rgb * scale,
      hemi: self.hemi * scale,
      sun: self.sun * scale,
    }
  }

  /// A term written to and read back from `fixed16<16>`: floored to a 32767th of 16, clamped to 16 bits.
  pub fn get_fixed(value: f32) -> f32 {
    let stored: i32 = ((value * 32767.0 / Self::FIXED_RANGE).floor() as i32).clamp(-32768, 32767);

    (stored as f32 / 32767.0) * Self::FIXED_RANGE
  }

  /// A term as a byte, `u8_clr`: floored to a 255th, clamped.
  pub fn get_byte(value: f32) -> u8 {
    ((value * 255.0).floor() as i32).clamp(0, 255) as u8
  }
}
