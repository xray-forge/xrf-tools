use image::RgbaImage;
use xrf_dds::DdsMipChain;
use xrf_error::XrfResult;
use xrf_thm::{ThmTextureFlag, ThmTextureParamChunk};

/// How a descriptor fades its mip chain towards a colour, an alpha, or both, the further a level is from the base.
///
/// The SDK carries out a fade two ways. For the `Advanced` filter it fades the chain it reduces itself
/// ([`crate::BuildTextureAdvancedChain`]); for every other filter it hands the same fields to nvDXT, whose fade
/// [`Self::fade_chain`] reproduces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildTextureFade {
  /// `fade_color` as the descriptor stores it, `0xAARRGGBB`; its alpha is the target of an alpha fade.
  pub fade_color: u32,
  /// Percent of the way each faded level moves towards the fade colour, `fade_amount`.
  pub fade_amount: u32,
  /// The first level below the base that fades, `fade_delay`; zero and one both fade from the first.
  pub fade_delay: u8,
  pub is_fading_color: bool,
  pub is_fading_alpha: bool,
}

impl BuildTextureFade {
  /// No fade at all, which is what a bump pair's chain carries whatever its descriptor says.
  pub const NONE: Self = Self {
    fade_color: 0,
    fade_amount: 0,
    fade_delay: 0,
    is_fading_color: false,
    is_fading_alpha: false,
  };

  /// The fade a descriptor asks for, from its texture parameters and the separate delay chunk.
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

  pub fn is_fading(&self) -> bool {
    self.is_fading_color || self.is_fading_alpha
  }

  /// How far one faded level moves towards the fade colour from the level it was reduced from, worked as
  /// `DXTCompressImage` works it: the complement of a clamped complement, and nothing at all when nothing fades.
  pub fn get_step(&self) -> f32 {
    if self.is_fading() {
      1.0 - (1.0 - self.fade_amount as f32 / 100.0).clamp(0.0, 1.0)
    } else {
      0.0
    }
  }

  /// Whether `level`, counted from the base, is one the fade reaches.
  pub fn is_reaching(&self, level: u32) -> bool {
    level >= 1 && level >= u32::from(self.fade_delay)
  }

  /// Fades a chain reduced from its base the way nvDXT fades one, which `DXTCompressImage` leaves to it for every filter
  /// but `Advanced`.
  ///
  /// nvDXT steps each faded level [`Self::get_step`] of the way towards the fade colour from the faded level above, so
  /// a level `k` steps past the delay has moved `1 - (1 - step)^k` of the way. Since a filter leaves a flat colour
  /// unchanged, that is the same as blending the unfaded level by that much, which is how it is worked here. Unlike
  /// the `Advanced` chain, the colour is not crossed, since `DXTCompressImage` hands nvDXT its channels by name, and
  /// alpha is not raised.
  ///
  /// The closed library's arithmetic is inferred, not ported: rebuilt from their shipped base level, 109 vanilla
  /// descriptors that fade over a Kaiser or box chain match their shipped levels within DXT noise this way and no other
  /// schedule tried fits better.
  ///
  /// # Errors
  ///
  /// Returns an error when the faded levels cannot be gathered back into a chain, which they always can, since a fade
  /// changes no level's size.
  pub fn fade_chain(&self, chain: DdsMipChain) -> XrfResult<DdsMipChain> {
    if !self.is_fading() {
      return Ok(chain);
    }

    let step: f32 = self.get_step();
    let [alpha, red, green, blue]: [u8; 4] = self.fade_color.to_be_bytes();
    let target: [f32; 4] = [f32::from(red), f32::from(green), f32::from(blue), f32::from(alpha)];
    let is_faded: [bool; 4] = [
      self.is_fading_color,
      self.is_fading_color,
      self.is_fading_color,
      self.is_fading_alpha,
    ];
    let mut steps: i32 = 0;
    let mut levels: Vec<RgbaImage> = Vec::with_capacity(chain.levels().len());

    for (level, image) in (0u32..).zip(chain.levels()) {
      if self.is_reaching(level) {
        steps += 1;
      }

      let blend: f32 = 1.0 - (1.0 - step).powi(steps);
      let mut faded: RgbaImage = image.clone();

      if blend > 0.0 {
        for texel in faded.pixels_mut() {
          for channel in (0..4).filter(|channel| is_faded[*channel]) {
            let value: f32 = f32::from(texel.0[channel]) * (1.0 - blend) + target[channel] * blend;

            texel.0[channel] = value.round().clamp(0.0, 255.0) as u8;
          }
        }
      }

      levels.push(faded);
    }

    DdsMipChain::from_levels(levels)
  }
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};
  use xrf_dds::{DdsMipChain, DdsMipFilter, DdsMipmaps};
  use xrf_error::XrfResult;

  use super::BuildTextureFade;

  /// A fade of both colour and alpha.
  fn fade(fade_amount: u32, fade_delay: u8) -> BuildTextureFade {
    BuildTextureFade {
      // Alpha 0x40, red 0xFF, green 0x80, blue 0x00.
      fade_color: 0x40FF_8000,
      fade_amount,
      fade_delay,
      is_fading_color: true,
      is_fading_alpha: true,
    }
  }

  fn flat(size: u32) -> XrfResult<DdsMipChain> {
    DdsMipChain::build(
      RgbaImage::from_pixel(size, size, Rgba([100, 100, 100, 200])),
      DdsMipmaps::Filtered(DdsMipFilter::Box),
    )
  }

  fn texel(chain: &DdsMipChain, level: usize) -> [u8; 4] {
    chain.levels()[level].get_pixel(0, 0).0
  }

  #[test]
  fn compounds_each_step_from_the_delayed_level_on() -> XrfResult {
    // Half way from level 2: level 1 is untouched, level 2 half way, level 3 three quarters, all rounded, with the
    // colour in its own channels.
    let colour: BuildTextureFade = BuildTextureFade {
      is_fading_alpha: false,
      ..fade(50, 2)
    };
    let faded: DdsMipChain = colour.fade_chain(flat(8)?)?;

    assert_eq!(texel(&faded, 0), [100, 100, 100, 200]);
    assert_eq!(texel(&faded, 1), [100, 100, 100, 200]);
    assert_eq!(texel(&faded, 2), [178, 114, 50, 200]);
    assert_eq!(texel(&faded, 3), [216, 121, 25, 200]);

    Ok(())
  }

  #[test]
  fn a_delay_of_zero_fades_from_the_first_level_like_one() -> XrfResult {
    let early: DdsMipChain = fade(25, 0).fade_chain(flat(4)?)?;
    let first: DdsMipChain = fade(25, 1).fade_chain(flat(4)?)?;

    assert_eq!(texel(&early, 0), [100, 100, 100, 200]);
    assert_eq!(early.levels(), first.levels());
    // Alpha a quarter of the way from 200 to 0x40.
    assert_eq!(texel(&early, 1)[3], 166);

    Ok(())
  }

  #[test]
  fn fades_only_the_channels_the_descriptor_names() -> XrfResult {
    let alpha: DdsMipChain = BuildTextureFade {
      is_fading_color: false,
      ..fade(100, 0)
    }
    .fade_chain(flat(2)?)?;

    assert_eq!(texel(&alpha, 1), [100, 100, 100, 0x40]);
    assert_eq!(BuildTextureFade::NONE.fade_chain(flat(2)?)?.levels(), flat(2)?.levels());

    Ok(())
  }
}
