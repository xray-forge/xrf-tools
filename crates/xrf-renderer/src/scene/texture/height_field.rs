//! A height field integrated from a tiling normal map (Frankot and Chellappa, "A method for enforcing integrability in
//! shape from shading algorithms", 1988): the normals' slopes projected onto the nearest integrable surface in the
//! Fourier domain, which a tiling texture's wrap makes exact. Where a texture brings no height of its own, its bump is
//! the only account of its relief there is.

use std::f32::consts::TAU;

/// The least a normal may face out of its surface before its slope is held, so a texel lying flat along it cannot make
/// the slope run away.
const LEAST_FACING: f32 = 0.2;
/// The share of the heights left out at each end when they are scaled into the texture's range, so a lone pit or spike
/// does not flatten the rest.
const OUTLIER_SHARE: f32 = 0.01;

/// Waves across the texture below which relief is faded out of a bump's heights: a normal map's slight bias over its
/// width, which no eye reads in its shading, integrates to swells across the whole tile that parallax would raise.
pub const BUMP_LOW_CUT: f32 = 8.0;
/// Texels a wave of relief spans at least before it is faded out of a bump's heights: the grain a normal map shades a
/// texel at a time raises, under parallax, a fur of thin columns at a glance along the ground.
pub const BUMP_GRAIN_TEXELS: f32 = 16.0;

/// The heights under a tiling normal map `width` by `height` texels, both powers of two, each normal in tangent space
/// (`x` along the texture's width, `y` along its height, `z` out of it): in texels, their mean at nought, relief of
/// fewer than about `low_cut` waves across the texture and of more than about `high_cut` faded out (neither at nought).
/// A normal map's `y` may run either way down the texture; the one whose slopes are integrable is taken.
pub fn integrate_normals(
  width: usize,
  height: usize,
  normals: &[[f32; 3]],
  (low_cut, high_cut): (f32, f32),
) -> Vec<f32> {
  let slopes: Vec<(f32, f32)> = normals
    .iter()
    .map(|&[x, y, z]| {
      let facing: f32 = z.max(LEAST_FACING);

      (-x / facing, -y / facing)
    })
    .collect();
  let sign: f32 = if get_curl(width, height, &slopes, 1.0) <= get_curl(width, height, &slopes, -1.0) {
    1.0
  } else {
    -1.0
  };
  let mut slope_x: (Vec<f32>, Vec<f32>) = (slopes.iter().map(|it| it.0).collect(), vec![0.0; slopes.len()]);
  let mut slope_y: (Vec<f32>, Vec<f32>) = (slopes.iter().map(|it| it.1 * sign).collect(), vec![0.0; slopes.len()]);

  transform_2d(width, height, (&mut slope_x.0, &mut slope_x.1), false);
  transform_2d(width, height, (&mut slope_y.0, &mut slope_y.1), false);

  // The forward difference's own frequency response, so the heights' differences give back the slopes exactly.
  let mut surface: (Vec<f32>, Vec<f32>) = (vec![0.0; slopes.len()], vec![0.0; slopes.len()]);

  for row in 0..height {
    let (across_y, along_y): (f32, f32) = difference(row, height);
    let waves_y: f32 = to_waves(row, height);

    for column in 0..width {
      let index: usize = row * width + column;

      if index == 0 {
        continue;
      }

      let (across_x, along_x): (f32, f32) = difference(column, width);
      let weight: f32 = across_x * across_x + along_x * along_x + across_y * across_y + along_y * along_y;

      if weight <= f32::EPSILON {
        continue;
      }

      // conj(d_x) P + conj(d_y) Q over |d_x|² + |d_y|², the broadest and the finest waves faded by Gaussian passes.
      let (px, py) = (slope_x.0[index], slope_x.1[index]);
      let (qx, qy) = (slope_y.0[index], slope_y.1[index]);
      let waves: f32 = to_waves(column, width).hypot(waves_y);
      let broad: f32 = if low_cut > 0.0 {
        1.0 - (-(waves / low_cut).powi(2)).exp()
      } else {
        1.0
      };
      let fine: f32 = if high_cut > 0.0 {
        (-(waves / high_cut).powi(2)).exp()
      } else {
        1.0
      };
      let kept: f32 = broad * fine;

      surface.0[index] = (across_x * px + along_x * py + across_y * qx + along_y * qy) / weight * kept;
      surface.1[index] = (across_x * py - along_x * px + across_y * qy - along_y * qx) / weight * kept;
    }
  }

  transform_2d(width, height, (&mut surface.0, &mut surface.1), true);

  surface.0
}

/// Heights scaled into nought to one about a half: the mean at a half, the larger of the farthest above and below it,
/// short of `OUTLIER_SHARE` at each end, reaching nought or one.
pub fn to_unit_heights(heights: &[f32]) -> Vec<f32> {
  let mean: f32 = heights.iter().sum::<f32>() / heights.len().max(1) as f32;
  // Every sixteenth height is enough to find where the ends lie.
  let mut sampled: Vec<f32> = heights.iter().step_by(16).map(|it| it - mean).collect();

  sampled.sort_by(f32::total_cmp);

  let last: usize = sampled.len().saturating_sub(1);
  let low: f32 = sampled
    .get((last as f32 * OUTLIER_SHARE) as usize)
    .copied()
    .unwrap_or(0.0);
  let high: f32 = sampled
    .get((last as f32 * (1.0 - OUTLIER_SHARE)) as usize)
    .copied()
    .unwrap_or(0.0);
  let reach: f32 = (-low).max(high).max(1e-6);

  heights
    .iter()
    .map(|it| (0.5 + (it - mean) * 0.5 / reach).clamp(0.0, 1.0))
    .collect()
}

/// How far slopes `x` and `y`, `y` taken with `sign`, are from any surface's: the sum of their curl's squares over the
/// wrapped texture.
fn get_curl(width: usize, height: usize, slopes: &[(f32, f32)], sign: f32) -> f32 {
  let at = |column: usize, row: usize| slopes[(row % height) * width + column % width];
  let mut sum: f32 = 0.0;

  for row in 0..height {
    for column in 0..width {
      let down: f32 = at(column, row + 1).0 - at(column, row + height - 1).0;
      let across: f32 = (at(column + 1, row).1 - at(column + width - 1, row).1) * sign;

      sum += (down - across) * (down - across);
    }
  }

  sum
}

/// Waves across the texture frequency `index` of `size` stands for, either way.
fn to_waves(index: usize, size: usize) -> f32 {
  index.min(size - index) as f32
}

/// The forward difference's response at frequency `index` of `size`: `e^(iω) - 1`, as its real and imaginary parts.
fn difference(index: usize, size: usize) -> (f32, f32) {
  let turn: f32 = TAU * index as f32 / size as f32;

  (turn.cos() - 1.0, turn.sin())
}

/// A two dimensional discrete Fourier transform in place, `width` by `height`, both powers of two: rows, then columns;
/// the inverse scaled by the texel count.
fn transform_2d(width: usize, height: usize, (real, imaginary): (&mut [f32], &mut [f32]), is_inverse: bool) {
  let across: Vec<(f32, f32)> = create_twiddles(width, is_inverse);
  let down: Vec<(f32, f32)> = create_twiddles(height, is_inverse);

  for row in 0..height {
    let span = row * width..(row + 1) * width;

    transform(&mut real[span.clone()], &mut imaginary[span], &across);
  }

  let (mut column_real, mut column_imaginary): (Vec<f32>, Vec<f32>) = (vec![0.0; height], vec![0.0; height]);

  for column in 0..width {
    for row in 0..height {
      column_real[row] = real[row * width + column];
      column_imaginary[row] = imaginary[row * width + column];
    }

    transform(&mut column_real, &mut column_imaginary, &down);

    for row in 0..height {
      real[row * width + column] = column_real[row];
      imaginary[row * width + column] = column_imaginary[row];
    }
  }

  if is_inverse {
    let scale: f32 = 1.0 / (width * height) as f32;

    real.iter_mut().for_each(|it| *it *= scale);
    imaginary.iter_mut().for_each(|it| *it *= scale);
  }
}

/// The turns a transform of `size` multiplies by, `e^(∓2πik/size)` for the first half of `k`, as cosine and sine.
fn create_twiddles(size: usize, is_inverse: bool) -> Vec<(f32, f32)> {
  let direction: f32 = if is_inverse { 1.0 } else { -1.0 };

  (0..size / 2)
    .map(|index| {
      let (sine, cosine): (f32, f32) = (direction * TAU * index as f32 / size as f32).sin_cos();

      (cosine, sine)
    })
    .collect()
}

/// A one dimensional radix-2 Cooley-Tukey transform in place, unscaled, by the `twiddles` of its length, a power of two.
fn transform(real: &mut [f32], imaginary: &mut [f32], twiddles: &[(f32, f32)]) {
  let size: usize = real.len();
  let bits: u32 = size.trailing_zeros();

  for index in 0..size {
    let reversed: usize = index.reverse_bits() >> (usize::BITS - bits);

    if reversed > index {
      real.swap(index, reversed);
      imaginary.swap(index, reversed);
    }
  }

  let mut span: usize = 2;

  while span <= size {
    let half: usize = span / 2;
    let stride: usize = size / span;

    for start in (0..size).step_by(span) {
      for offset in 0..half {
        let (twiddle_real, twiddle_imaginary): (f32, f32) = twiddles[offset * stride];
        let (even, odd): (usize, usize) = (start + offset, start + offset + half);
        let odd_real: f32 = real[odd] * twiddle_real - imaginary[odd] * twiddle_imaginary;
        let odd_imaginary: f32 = real[odd] * twiddle_imaginary + imaginary[odd] * twiddle_real;

        real[odd] = real[even] - odd_real;
        imaginary[odd] = imaginary[even] - odd_imaginary;
        real[even] += odd_real;
        imaginary[even] += odd_imaginary;
      }
    }

    span *= 2;
  }
}
