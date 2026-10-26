use image::{Rgba, RgbaImage};
use xrf_dds::DdsMipChain;
use xrf_error::{XrfError, XrfResult};
use xrf_thm::{ThmTextureFlag, ThmTextureParamChunk};

/// The mip chain the SDK reduces itself for a descriptor whose filter is `Advanced`, rather than leaving it to nvDXT:
/// `DXTCompressImage` with `Build32MipLevel` (`xray/trunk/xrDXT/DXT.cpp`).
///
/// Each level is the 2x2 average of the one above, faded towards `fade_color` once `fade_delay` levels are past, and
/// its alpha raised by an eighth, which keeps an alpha-tested surface from thinning away at a distance. The chain stops
/// when the shorter side reaches one texel, so an oblong texture carries fewer levels than a full chain.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildTextureAdvancedChain {
  /// `fade_color` as the descriptor stores it, `0xAARRGGBB`; its alpha is the target of an alpha fade.
  pub fade_color: u32,
  /// Percent of the way each faded level moves towards the fade colour, `fade_amount`.
  pub fade_amount: u32,
  /// Levels reduced before fading starts, `fade_delay`.
  pub fade_delay: u8,
  pub is_fading_color: bool,
  pub is_fading_alpha: bool,
}

impl BuildTextureAdvancedChain {
  /// The chain a descriptor asks for, its fade read from the texture parameters and the separate delay chunk.
  pub fn of(param: &ThmTextureParamChunk, fade_delay: Option<u8>) -> Self {
    Self {
      fade_color: param.fade_color,
      fade_amount: param.fade_amount,
      // `STextureParams` starts zeroed, so a descriptor without the chunk fades from the first level.
      fade_delay: fade_delay.unwrap_or(0),
      is_fading_color: param.flags.has(ThmTextureFlag::FadeToColor),
      is_fading_alpha: param.flags.has(ThmTextureFlag::FadeToAlpha),
    }
  }

  /// Reduces `base` level by level, each from the one above, as the SDK does.
  ///
  /// # Errors
  ///
  /// Returns an error when the base has no pixels, or a level to be halved has an odd side, which `Build32MipLevel`
  /// asserts against: its 2x2 average has no texel to pair the last one with.
  pub fn build(&self, base: &RgbaImage) -> XrfResult<DdsMipChain> {
    let (width, height): (u32, u32) = base.dimensions();

    if width == 0 || height == 0 {
      return Err(XrfError::new_texture_processing_error(
        "Cannot build a mip chain from an image with no pixels",
      ));
    }

    // `GetPowerOf2Plus1(min(w, h))`: the bit length of the shorter side.
    let count: u32 = width.min(height).ilog2() + 1;
    let blend: f32 = self.get_blend();
    let mut levels: Vec<RgbaImage> = vec![base.clone()];

    for level in 1..count {
      let above: &RgbaImage = levels.last().expect("the chain starts at its base");

      if !above.width().is_multiple_of(2) || !above.height().is_multiple_of(2) {
        return Err(XrfError::new_texture_processing_error(format!(
          "The 'Advanced' mip filter halves each level by 2x2 averages, and level {} of a {width}x{height} texture is \
           {}x{}, which has no pair for its last texel",
          level - 1,
          above.width(),
          above.height()
        )));
      }

      let reduced: RgbaImage = self.reduce(above, if level < u32::from(self.fade_delay) { 0.0 } else { blend });

      levels.push(reduced);
    }

    DdsMipChain::from_levels(levels)
  }

  /// How far a faded level moves towards the fade colour, worked as `DXTCompressImage` works it: the complement of a
  /// clamped complement, and nothing at all when the descriptor fades neither colour nor alpha.
  fn get_blend(&self) -> f32 {
    let inverse_fade: f32 = (1.0 - self.fade_amount as f32 / 100.0).clamp(0.0, 1.0);
    let kept: f32 = if self.is_fading_color || self.is_fading_alpha {
      inverse_fade
    } else {
      1.0
    };

    1.0 - kept
  }

  /// One level of `Build32MipLevel`, half the size of `above`.
  ///
  /// The SDK names its bytes red to blue while a `D3DCOLOR` lies in memory blue to red, so it pulls the blue channel
  /// towards the fade colour's red and the red channel towards its blue. Kept, since that is what the shipped
  /// textures were built with; a grey fade, the usual one, cannot show it.
  fn reduce(&self, above: &RgbaImage, blend: f32) -> RgbaImage {
    let [alpha, red, green, blue]: [u8; 4] = self.fade_color.to_be_bytes();
    let target: [f32; 4] = [f32::from(blue), f32::from(green), f32::from(red), f32::from(alpha)];
    let keep: f32 = 1.0 - blend;

    RgbaImage::from_fn(above.width() / 2, above.height() / 2, |x, y| {
      let texels: [&Rgba<u8>; 4] = [
        above.get_pixel(2 * x, 2 * y),
        above.get_pixel(2 * x + 1, 2 * y),
        above.get_pixel(2 * x, 2 * y + 1),
        above.get_pixel(2 * x + 1, 2 * y + 1),
      ];
      let mut channels: [f32; 4] =
        [0, 1, 2, 3].map(|channel| texels.iter().map(|texel| u32::from(texel.0[channel])).sum::<u32>() as f32 / 4.0);

      if self.is_fading_color {
        for channel in 0..3 {
          channels[channel] = channels[channel] * keep + target[channel] * blend;
        }
      }

      if self.is_fading_alpha {
        channels[3] = channels[3] * keep + target[3] * blend;
      }

      channels[3] += channels[3] / 8.0;

      // `int(c)` then `clamp(c, 0, 255)`: truncated, not rounded.
      Rgba(channels.map(|value| (value as i32).clamp(0, 255) as u8))
    })
  }
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};
  use xrf_dds::DdsMipChain;
  use xrf_error::XrfResult;

  use super::BuildTextureAdvancedChain;

  fn chain(fade_amount: u32, fade_delay: u8, is_fading: bool) -> BuildTextureAdvancedChain {
    BuildTextureAdvancedChain {
      // Alpha 0x40, red 0xFF, green 0x80, blue 0x00.
      fade_color: 0x40FF_8000,
      fade_amount,
      fade_delay,
      is_fading_color: is_fading,
      is_fading_alpha: is_fading,
    }
  }

  fn texel(chain: &DdsMipChain, level: usize) -> [u8; 4] {
    chain.levels()[level].get_pixel(0, 0).0
  }

  #[test]
  fn averages_each_quad_and_raises_its_alpha_by_an_eighth() -> XrfResult {
    // Four texels averaging to `(20, 40, 60, 100)`; the alpha comes out `100 + 100 / 8`, truncated to 112.
    let base: RgbaImage = RgbaImage::from_fn(2, 2, |x, y| {
      let step: u8 = (x + 2 * y) as u8;

      Rgba([14 + 4 * step, 34 + 4 * step, 54 + 4 * step, 94 + 4 * step])
    });

    assert_eq!(texel(&chain(0, 0, false).build(&base)?, 1), [20, 40, 60, 112]);

    Ok(())
  }

  #[test]
  fn stops_when_the_shorter_side_reaches_one() -> XrfResult {
    let chain: DdsMipChain = chain(0, 0, false).build(&RgbaImage::new(16, 4))?;

    assert_eq!(
      chain.levels().iter().map(RgbaImage::dimensions).collect::<Vec<_>>(),
      [(16, 4), (8, 2), (4, 1)]
    );

    Ok(())
  }

  #[test]
  fn fades_from_the_delayed_level_on_with_red_and_blue_crossed() -> XrfResult {
    // Half way towards the fade colour from level 2: level 1 is only averaged, level 2 moves half the way. The red
    // channel moves towards the colour's blue (0) and the blue channel towards its red (255), as the SDK's byte order
    // has it.
    let base: RgbaImage = RgbaImage::from_pixel(4, 4, Rgba([100, 100, 100, 64]));
    let faded: DdsMipChain = chain(50, 2, true).build(&base)?;

    assert_eq!(texel(&faded, 1), [100, 100, 100, 72]);
    // Alpha: `(72 * 0.5 + 64 * 0.5) * 9 / 8` = 76.5, truncated.
    assert_eq!(texel(&faded, 2), [50, 114, 177, 76]);

    Ok(())
  }

  #[test]
  fn refuses_a_level_it_cannot_halve() {
    assert!(chain(0, 0, false).build(&RgbaImage::new(6, 6)).is_err());
  }
}
