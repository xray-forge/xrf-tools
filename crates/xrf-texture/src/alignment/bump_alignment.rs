use image::imageops::{self, FilterType};
use image::{ImageBuffer, Luma};
use serde::Serialize;
use xrf_dds::{DdsFile, RgbaImage};
use xrf_error::XrfResult;

use crate::alignment::BumpAlignmentVerdict;

/// One value a texel, as both textures are brought to before they are compared.
type Field = ImageBuffer<Luma<f32>, Vec<f32>>;

/// Whether a bump belongs to its diffuse: how closely the bump's edges follow the diffuse's, against how closely they
/// would by chance.
///
/// A set authored together draws its relief from the picture, so a crack, a seam, a rivet or an atlas island's outline
/// is an edge in both at the same texels. A diffuse from one set over a bump from another shares nothing but what any
/// two textures do, which is what the bump shifted along the tile measures. Edges are compared rather than the bump's
/// slopes: an atlas's bump often lies flat along the outlines where its diffuse changes most.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpAlignment {
  /// Texels a side of the diffuse's base level.
  pub diffuse_size: [u32; 2],
  /// Texels a side of the bump's base level.
  pub bump_size: [u32; 2],
  /// The correlation of the diffuse's edges with the bump's, texel for texel; `None` where one carries nothing to
  /// compare.
  pub correlation: Option<f32>,
  /// The highest correlation of the bump shifted along the tile, what the two share by chance.
  pub chance: Option<f32>,
  /// How far the correlation rises over chance, which the verdict judges.
  pub lift: Option<f32>,
  pub verdict: BumpAlignmentVerdict,
}

impl BumpAlignment {
  /// Texels the longer side of the compared levels holds at most: at 256 a bump's fine relief, corrugation or an
  /// atlas island's bevel, averaged away, and 96 of stock Anomaly's sets read as mismatched that read as aligned here.
  pub const MEASURED_SIDE: u32 = 512;
  /// Shifts along the tile that measure chance, their highest correlation taken: the first points of Roberts' R2
  /// sequence, spread evenly over the tile and never a whole fraction of it, so a pattern repeating a whole number of
  /// times across the tile does not line up with itself.
  pub const CHANCE_SHIFTS: u32 = 8;
  /// The spread a field's values stay under when it carries nothing to compare.
  const FEATURELESS_SPREAD: f32 = 1e-3;

  /// Compares a diffuse with its bump, each read at its largest level within [`Self::MEASURED_SIDE`].
  ///
  /// # Errors
  ///
  /// Returns an error when either level does not decode.
  pub fn read(diffuse: &DdsFile, bump: &DdsFile) -> XrfResult<Self> {
    let (diffuse_metadata, bump_metadata) = (diffuse.metadata(), bump.metadata());

    Ok(Self::measure(
      [diffuse_metadata.width, diffuse_metadata.height],
      &diffuse.decode_rgba(diffuse_metadata.get_level_within(Self::MEASURED_SIDE))?,
      [bump_metadata.width, bump_metadata.height],
      &bump.decode_rgba(bump_metadata.get_level_within(Self::MEASURED_SIDE))?,
    ))
  }

  /// Compares decoded levels of a diffuse and a bump whose base levels are `diffuse_size` and `bump_size`.
  ///
  /// Both are sampled over the same texture coordinates whatever their proportions, so they are compared at the
  /// coarser of the two along each side, neither read finer than it was drawn.
  pub fn measure(diffuse_size: [u32; 2], diffuse: &RgbaImage, bump_size: [u32; 2], bump: &RgbaImage) -> Self {
    let (width, height): (u32, u32) = (diffuse.width().min(bump.width()), diffuse.height().min(bump.height()));
    let featureless = || Self {
      diffuse_size,
      bump_size,
      correlation: None,
      chance: None,
      lift: None,
      verdict: BumpAlignmentVerdict::Featureless,
    };

    // A few texels a side hold no edge a shift could move off.
    if width < 4 || height < 4 {
      return featureless();
    }

    let edges: Field = Self::edges_of(Self::colour_of(diffuse), width, height);
    let relief: Field = Self::edges_of(Self::normal_of(bump), width, height);
    let Some(correlation) = Self::correlation(&edges, &relief, (0, 0)) else {
      return featureless();
    };
    let chance: f32 = (1..=Self::CHANCE_SHIFTS)
      .filter_map(|index| {
        // R2's generator, the plastic number's inverse and its square.
        let (across, down): (f64, f64) = (
          (f64::from(index) * 0.754_877_666).fract(),
          (f64::from(index) * 0.569_840_291).fract(),
        );
        let shift: (u32, u32) = (
          ((f64::from(width) * across) as u32).max(1),
          ((f64::from(height) * down) as u32).max(1),
        );

        Self::correlation(&edges, &relief, shift)
      })
      .fold(f32::MIN, f32::max);
    let lift: f32 = correlation - chance;

    Self {
      diffuse_size,
      bump_size,
      correlation: Some(correlation),
      chance: Some(chance),
      lift: Some(lift),
      verdict: BumpAlignmentVerdict::of(lift),
    }
  }

  /// The diffuse's colour as it is stored, a channel each: an edge is where the picture changes, in hue as much as in
  /// lightness, whatever light it decodes to.
  fn colour_of(image: &RgbaImage) -> [Field; 3] {
    [0, 1, 2].map(|channel| {
      Field::from_fn(image.width(), image.height(), |x, y| {
        Luma([f32::from(image.get_pixel(x, y)[channel]) / 255.0])
      })
    })
  }

  /// The bump's normal, a component each: stored reversed behind the gloss, `x` in alpha, `y` in blue and `z` in
  /// green. The gloss is left out, being no part of the relief.
  fn normal_of(image: &RgbaImage) -> [Field; 3] {
    [3, 2, 1].map(|channel| {
      Field::from_fn(image.width(), image.height(), |x, y| {
        Luma([f32::from(image.get_pixel(x, y)[channel]) / 255.0 * 2.0 - 1.0])
      })
    })
  }

  /// How sharply a picture of `channels` changes at each texel, at `width` by `height`: the length of every channel's
  /// gradient together.
  fn edges_of(channels: [Field; 3], width: u32, height: u32) -> Field {
    let gradients: [Field; 3] = channels.map(|channel| Self::gradient_of(&Self::resized(channel, width, height)));

    Field::from_fn(width, height, |x, y| {
      Luma([gradients
        .iter()
        .map(|gradient| gradient.get_pixel(x, y)[0].powi(2))
        .sum::<f32>()
        .sqrt()])
    })
  }

  /// The length of a field's gradient, by Sobel's kernels across the tile's wrapped edges.
  fn gradient_of(field: &Field) -> Field {
    let (width, height): (u32, u32) = field.dimensions();
    let at = |x: u32, y: u32, dx: i64, dy: i64| {
      let column: u32 = (i64::from(x) + dx).rem_euclid(i64::from(width)) as u32;
      let row: u32 = (i64::from(y) + dy).rem_euclid(i64::from(height)) as u32;

      field.get_pixel(column, row)[0]
    };

    Field::from_fn(width, height, |x, y| {
      let across: f32 = at(x, y, 1, -1) + 2.0 * at(x, y, 1, 0) + at(x, y, 1, 1)
        - at(x, y, -1, -1)
        - 2.0 * at(x, y, -1, 0)
        - at(x, y, -1, 1);
      let down: f32 = at(x, y, -1, 1) + 2.0 * at(x, y, 0, 1) + at(x, y, 1, 1)
        - at(x, y, -1, -1)
        - 2.0 * at(x, y, 0, -1)
        - at(x, y, 1, -1);

      Luma([across.hypot(down)])
    })
  }

  fn resized(field: Field, width: u32, height: u32) -> Field {
    if field.dimensions() == (width, height) {
      return field;
    }

    imageops::resize(&field, width, height, FilterType::Triangle)
  }

  /// Pearson's correlation of two fields of one size, the second read `shift` texels along the tile; `None` where
  /// either is as good as constant.
  fn correlation(first: &Field, second: &Field, shift: (u32, u32)) -> Option<f32> {
    let (width, height): (u32, u32) = first.dimensions();
    let count: f64 = f64::from(width) * f64::from(height);
    let mean = |field: &Field| field.pixels().map(|it| f64::from(it[0])).sum::<f64>() / count;
    let (first_mean, second_mean): (f64, f64) = (mean(first), mean(second));
    let (mut covariance, mut first_variance, mut second_variance): (f64, f64, f64) = (0.0, 0.0, 0.0);

    for (x, y, value) in first.enumerate_pixels() {
      let a: f64 = f64::from(value[0]) - first_mean;
      let b: f64 = f64::from(second.get_pixel((x + shift.0) % width, (y + shift.1) % height)[0]) - second_mean;

      covariance += a * b;
      first_variance += a * a;
      second_variance += b * b;
    }

    let spread: f64 = f64::from(Self::FEATURELESS_SPREAD).powi(2) * count;

    if first_variance < spread || second_variance < spread {
      return None;
    }

    Some((covariance / (first_variance * second_variance).sqrt()) as f32)
  }
}
