use image::imageops::{self, FilterType};
use image::{ImageBuffer, Luma};
use serde::Serialize;
use xrf_dds::RgbaImage;

use crate::alignment::{BumpAlignmentLevel, BumpAlignmentVerdict};

/// One value a texel, as both textures are brought to before they are compared: `f32` storage for a texture-sized
/// buffer, the correlation over it taken in `f64`.
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
  pub correlation: Option<f64>,
  /// The highest correlation of the bump shifted along the tile, what the two share by chance.
  pub chance: Option<f64>,
  /// How far the correlation rises over chance, which the verdict judges.
  pub lift: Option<f64>,
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
  const FEATURELESS_SPREAD: f64 = 1e-3;
  /// Correlations a comparison takes: the tile as it lies, and each shift measuring chance.
  const SHIFTS: usize = Self::CHANCE_SHIFTS as usize + 1;

  /// Compares a diffuse with its bump, each read at its largest level within [`Self::MEASURED_SIDE`].
  ///
  /// A caller reading many pairs reads each with [`BumpAlignmentLevel::read`] and lets the file go before comparing
  /// them: the comparison is most of the time, and the whole files need not be held through it.
  pub fn compare(diffuse: &BumpAlignmentLevel, bump: &BumpAlignmentLevel) -> Self {
    Self::measure(diffuse.base_size, &diffuse.image, bump.base_size, &bump.image)
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

    let edges: Field = Self::colour_edges(diffuse, width, height);
    let relief: Field = Self::normal_edges(bump, width, height);
    let mut shifts: [(u32, u32); Self::SHIFTS] = [(0, 0); Self::SHIFTS];

    // The tile as it lies first, then the shifts that measure chance.
    for (index, shift) in (1..=Self::CHANCE_SHIFTS).zip(&mut shifts[1..]) {
      // R2's generator, the plastic number's inverse and its square.
      let (across, down): (f64, f64) = (
        (f64::from(index) * 0.754_877_666).fract(),
        (f64::from(index) * 0.569_840_291).fract(),
      );

      *shift = (
        ((f64::from(width) * across) as u32).max(1),
        ((f64::from(height) * down) as u32).max(1),
      );
    }

    let [aligned, shifted @ ..] = Self::correlations(&edges, &relief, shifts);
    let Some(correlation) = aligned else {
      return featureless();
    };
    let chance: f64 = shifted.into_iter().flatten().fold(f64::MIN, f64::max);
    let lift: f64 = correlation - chance;

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
  fn colour_edges(image: &RgbaImage, width: u32, height: u32) -> Field {
    Self::edges_of(image, [0, 1, 2], |value| value / 255.0, width, height)
  }

  /// The bump's normal, a component each: stored reversed behind the gloss, `x` in alpha, `y` in blue and `z` in
  /// green. The gloss is left out, being no part of the relief.
  fn normal_edges(image: &RgbaImage, width: u32, height: u32) -> Field {
    Self::edges_of(image, [3, 2, 1], |value| value / 255.0 * 2.0 - 1.0, width, height)
  }

  /// How sharply the picture `channels` of `image` hold changes at each texel, each byte read as `value` answers for
  /// it, at `width` by `height`: the length of every channel's gradient together.
  ///
  /// A level already that size is read where it lies, through a table of what each byte stands for; only one to be
  /// resized is first spread into a field a channel.
  fn edges_of(image: &RgbaImage, channels: [usize; 3], value: impl Fn(f32) -> f32, width: u32, height: u32) -> Field {
    if image.dimensions() == (width, height) {
      let mut table: [f32; 256] = [0.0; 256];

      for (byte, entry) in (0..=u8::MAX).zip(&mut table) {
        *entry = value(f32::from(byte));
      }

      let texels: &[u8] = image.as_raw();

      return Self::edges_from(width, height, |channel, texel| {
        table[usize::from(texels[texel * 4 + channels[channel]])]
      });
    }

    let planes: [Field; 3] = channels.map(|channel| {
      let mut plane: Field = Field::new(image.width(), image.height());
      let values: &mut [f32] = &mut plane;

      for (texel, pixel) in values.iter_mut().zip(image.as_raw().as_chunks::<4>().0) {
        *texel = value(f32::from(pixel[channel]));
      }

      imageops::resize(&plane, width, height, FilterType::Triangle)
    });

    Self::edges_from(width, height, |channel, texel| planes[channel].as_raw()[texel])
  }

  /// The length of three channels' gradients together at each texel, `sample` answering a channel's value at a texel
  /// index; each gradient by Sobel's kernels across the tile's wrapped edges.
  fn edges_from(width: u32, height: u32, sample: impl Fn(usize, usize) -> f32) -> Field {
    let (columns, rows): (usize, usize) = (width as usize, height as usize);
    let mut edges: Field = Field::new(width, height);
    let values: &mut [f32] = &mut edges;

    for (y, row) in values.chunks_exact_mut(columns).enumerate() {
      // The first texel of the row above, this one and the one below.
      let above: usize = Self::wrapped(y + rows - 1, rows) * columns;
      let middle: usize = y * columns;
      let below: usize = Self::wrapped(y + 1, rows) * columns;

      for (x, edge) in row.iter_mut().enumerate() {
        let (left, right): (usize, usize) = (Self::wrapped(x + columns - 1, columns), Self::wrapped(x + 1, columns));
        let gradients: [f32; 3] = std::array::from_fn(|channel| {
          let at = |row_start: usize, column: usize| sample(channel, row_start + column);
          let across: f32 = at(above, right) + 2.0 * at(middle, right) + at(below, right)
            - at(above, left)
            - 2.0 * at(middle, left)
            - at(below, left);
          let down: f32 = at(below, left) + 2.0 * at(below, x) + at(below, right)
            - at(above, left)
            - 2.0 * at(above, x)
            - at(above, right);

          across.hypot(down)
        });

        *edge = gradients.iter().map(|gradient| gradient.powi(2)).sum::<f32>().sqrt();
      }
    }

    edges
  }

  /// An index past the end of an axis `length` long, by less than a length, brought back from its start: the tile's
  /// wrap, without the division a remainder takes on every texel.
  fn wrapped(index: usize, length: usize) -> usize {
    if index >= length { index - length } else { index }
  }

  /// Pearson's correlation of two fields of one size at each of `shifts`, the second read that many texels along the
  /// tile; `None` where either is as good as constant.
  ///
  /// One pass serves every shift, each one's sums kept apart and taken texel by texel in the order a pass of its own
  /// would take them, so each answer is what correlating at that shift alone gives.
  fn correlations(first: &Field, second: &Field, shifts: [(u32, u32); Self::SHIFTS]) -> [Option<f64>; Self::SHIFTS] {
    let (width, height): (u32, u32) = first.dimensions();
    let count: f64 = f64::from(width) * f64::from(height);
    let mean = |field: &Field| field.pixels().map(|it| f64::from(it[0])).sum::<f64>() / count;
    let (first_mean, second_mean): (f64, f64) = (mean(first), mean(second));
    let mut first_variance: f64 = 0.0;
    let mut covariances: [f64; Self::SHIFTS] = [0.0; Self::SHIFTS];
    let mut second_variances: [f64; Self::SHIFTS] = [0.0; Self::SHIFTS];
    let (columns, rows): (usize, usize) = (width as usize, height as usize);
    let offsets: [(usize, usize); Self::SHIFTS] =
      shifts.map(|(across, down)| ((across % width) as usize, (down % height) as usize));
    let second_values: &[f32] = second.as_raw();

    for (y, first_row) in first.as_raw().chunks_exact(columns).enumerate() {
      let second_rows: [&[f32]; Self::SHIFTS] =
        offsets.map(|(_, down)| &second_values[Self::wrapped(y + down, rows) * columns..][..columns]);

      for (x, value) in first_row.iter().enumerate() {
        let a: f64 = f64::from(*value) - first_mean;

        first_variance += a * a;

        for (shift, ((across, _), second_row)) in offsets.iter().zip(&second_rows).enumerate() {
          let b: f64 = f64::from(second_row[Self::wrapped(x + across, columns)]) - second_mean;

          covariances[shift] += a * b;
          second_variances[shift] += b * b;
        }
      }
    }

    let spread: f64 = Self::FEATURELESS_SPREAD.powi(2) * count;

    std::array::from_fn(|shift| {
      if first_variance < spread || second_variances[shift] < spread {
        return None;
      }

      Some(covariances[shift] / (first_variance * second_variances[shift]).sqrt())
    })
  }
}
