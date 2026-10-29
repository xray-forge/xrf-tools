use serde::Serialize;
use xrf_dds::{DdsFile, RgbaImage};
use xrf_error::XrfResult;
use xrf_math::{luminance, srgb_decoded};

use crate::albedo::AlbedoVerdict;

/// How a diffuse texture's colours sit as albedo: the linear values the renderer lights at its physical levels, measured
/// over the texels a surface draws.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureAlbedo {
  /// Texels a side of the mip level measured.
  pub width: u32,
  pub height: u32,
  /// Texels measured: all of them, or a cut-out's that its alpha test keeps.
  pub texels: u32,
  /// The mean linear colour.
  pub mean: [f32; 3],
  /// The linear luminance a twentieth of the texels fall below.
  pub low: f32,
  /// The median linear luminance, which the verdict judges: a texture's typical colour, whatever a few texels do.
  pub median: f32,
  /// The linear luminance a twentieth of the texels rise above.
  pub high: f32,
  /// The share of texels darker than [`AlbedoVerdict::FLOOR`].
  pub dark_share: f32,
  /// The share of texels brighter than [`AlbedoVerdict::CEILING`].
  pub bright_share: f32,
  /// Stops between the brighter and the darker tenth of the texture's cells, an eighth of a side each: large-scale
  /// light and shade painted in widen it, as does an atlas of unlike materials.
  pub coarse_spread: f32,
  pub verdict: AlbedoVerdict,
}

impl TextureAlbedo {
  /// Texels the longer side of the measured level holds at most: a level coarser than the base reads the same
  /// statistics at a fraction of the decode.
  pub const MEASURED_SIDE: u32 = 512;
  /// Cells a side the coarse spread compares.
  pub const CELLS: u32 = 8;

  /// Measures a texture file at its largest level within [`Self::MEASURED_SIDE`], `None` where no texel passes.
  ///
  /// # Errors
  ///
  /// Returns an error when the level does not decode.
  pub fn read(file: &DdsFile, alpha_reference: Option<u8>) -> XrfResult<Option<Self>> {
    let level: u32 = file.metadata().get_level_within(Self::MEASURED_SIDE);

    Ok(Self::measure(&file.decode_rgba(level)?, alpha_reference))
  }

  /// Measures decoded texels, those whose alpha falls below `alpha_reference` left out as the alpha test kills them;
  /// `None` where none is left. Each channel is decoded as the renderer decodes a colour texture.
  pub fn measure(image: &RgbaImage, alpha_reference: Option<u8>) -> Option<Self> {
    let decode: [f32; 256] = std::array::from_fn(|value| srgb_decoded(value as f32 / 255.0));
    let (width, height): (u32, u32) = image.dimensions();
    let (columns, rows): (u32, u32) = (Self::CELLS.min(width), Self::CELLS.min(height));
    let mut cells: Vec<(f64, u32)> = vec![(0.0, 0); (columns * rows) as usize];
    let mut luminances: Vec<f32> = Vec::with_capacity((width * height) as usize);
    let mut sum: [f64; 3] = [0.0; 3];

    for (x, y, texel) in image.enumerate_pixels() {
      if alpha_reference.is_some_and(|reference| texel[3] < reference) {
        continue;
      }

      let colour: [f32; 3] = [
        decode[texel[0] as usize],
        decode[texel[1] as usize],
        decode[texel[2] as usize],
      ];
      let luminance: f32 = luminance(colour);
      let cell: &mut (f64, u32) = &mut cells[(y * rows / height * columns + x * columns / width) as usize];

      for (total, channel) in sum.iter_mut().zip(colour) {
        *total += f64::from(channel);
      }

      cell.0 += f64::from(luminance);
      cell.1 += 1;
      luminances.push(luminance);
    }

    if luminances.is_empty() {
      return None;
    }

    let texels: usize = luminances.len();
    let dark: usize = luminances.iter().filter(|it| **it < AlbedoVerdict::FLOOR).count();
    let bright: usize = luminances.iter().filter(|it| **it > AlbedoVerdict::CEILING).count();

    luminances.sort_unstable_by(f32::total_cmp);

    let at = |share: f32| luminances[((texels - 1) as f32 * share).round() as usize];
    let median: f32 = at(0.5);

    Some(Self {
      width,
      height,
      texels: texels as u32,
      mean: sum.map(|total| (total / texels as f64) as f32),
      low: at(0.05),
      median,
      high: at(0.95),
      dark_share: dark as f32 / texels as f32,
      bright_share: bright as f32 / texels as f32,
      coarse_spread: Self::spread_of(&cells, width * height / (columns * rows)),
      verdict: AlbedoVerdict::of(median),
    })
  }

  /// Stops between the brighter and the darker tenth of the cells, of those at least a quarter covered; none where
  /// fewer than four are.
  fn spread_of(cells: &[(f64, u32)], cell_texels: u32) -> f32 {
    let mut means: Vec<f32> = cells
      .iter()
      .filter(|(_, count)| *count * 4 >= cell_texels.max(1))
      .map(|(total, count)| (total / f64::from(*count)) as f32)
      .collect();

    if means.len() < 4 {
      return 0.0;
    }

    means.sort_unstable_by(f32::total_cmp);

    let at = |share: f32| means[((means.len() - 1) as f32 * share).round() as usize];

    (at(0.9).max(1e-4) / at(0.1).max(1e-4)).log2()
  }
}
