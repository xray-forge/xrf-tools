/// xrLC's image rescaler (`imf_Process`, `xrLC_Light/xrImage_Resampler.cpp`, Dale Schumacher's "Filtered Image
/// Rescaling"), with which RMS compression tries a chart at a smaller size: rows then columns, each output byte the sum
/// of its source bytes by their filter weights, not normalised, rounded and clamped, the fourth with half a step more,
/// a source pixel past an edge mirrored back.
pub(crate) enum LightmapResampler {
  /// `imf_filter`: the cubic `2|t|³ - 3|t|² + 1` within one pixel, which grows a chart back.
  Cubic,
  /// `imf_lanczos3`: Lanczos within three pixels, which shrinks a chart.
  Lanczos3,
}

/// One source pixel's weight in an output pixel.
struct Contribution {
  pixel: usize,
  weight: f32,
}

impl LightmapResampler {
  /// `pixels`, `width` by `height`, rescaled to `to_width` by `to_height`. Each side must be over one.
  pub(crate) fn resample(
    &self,
    pixels: &[[u8; 4]],
    width: u32,
    height: u32,
    to_width: u32,
    to_height: u32,
  ) -> Vec<[u8; 4]> {
    let (width, height, to_width, to_height) = (width as usize, height as usize, to_width as usize, to_height as usize);
    let columns: Vec<Vec<Contribution>> = self.get_contributions(width, to_width);
    let mut across: Vec<[u8; 4]> = vec![[0; 4]; to_width * height];

    for y in 0..height {
      let row: &[[u8; 4]] = &pixels[y * width..(y + 1) * width];

      for (x, contributions) in columns.iter().enumerate() {
        across[y * to_width + x] = Self::get_pixel(contributions, |pixel| row[pixel]);
      }
    }

    let rows: Vec<Vec<Contribution>> = self.get_contributions(height, to_height);
    let mut result: Vec<[u8; 4]> = vec![[0; 4]; to_width * to_height];

    for x in 0..to_width {
      for (y, contributions) in rows.iter().enumerate() {
        result[y * to_width + x] = Self::get_pixel(contributions, |pixel| across[pixel * to_width + x]);
      }
    }

    result
  }

  /// The filter's reach, in source pixels at full scale.
  fn get_support(&self) -> f32 {
    match self {
      Self::Cubic => 1.0,
      Self::Lanczos3 => 3.0,
    }
  }

  fn get_weight(&self, t: f32) -> f32 {
    let t: f32 = t.abs();

    match self {
      Self::Cubic => {
        if t < 1.0 {
          ((2.0 * f64::from(t) - 3.0) * f64::from(t) * f64::from(t) + 1.0) as f32
        } else {
          0.0
        }
      }
      Self::Lanczos3 => {
        if t < 3.0 {
          Self::sinc(t) * Self::sinc(t / 3.0)
        } else {
          0.0
        }
      }
    }
  }

  #[expect(
    clippy::approx_constant,
    reason = "the source's `3.1415926f`, a float below `PI`'s, which every weight is taken by"
  )]
  fn sinc(x: f32) -> f32 {
    let x: f32 = x * 3.141_592_6;

    if x != 0.0 { x.sin() / x } else { 1.0 }
  }

  /// Each output pixel's source pixels and weights along a side of `from` pixels rescaled to `to`.
  fn get_contributions(&self, from: usize, to: usize) -> Vec<Vec<Contribution>> {
    let scale: f32 = to as f32 / from as f32;
    let support: f32 = self.get_support();
    let (width, divisor) = if scale < 1.0 {
      (support / scale, 1.0 / scale)
    } else {
      (support, 1.0)
    };

    (0..to)
      .map(|output| {
        let center: f32 = output as f32 / scale;
        let (left, right) = ((center - width).ceil() as i64, (center + width).floor() as i64);

        (left..=right)
          .map(|source| {
            let distance: f32 = center - source as f32;
            let weight: f32 = if scale < 1.0 {
              self.get_weight(distance / divisor) / divisor
            } else {
              self.get_weight(distance)
            };
            let from: i64 = i64::try_from(from).unwrap_or(i64::MAX);
            let mirrored: i64 = if source < 0 {
              -source
            } else if source >= from {
              (from - source) + from - 1
            } else {
              source
            };

            Contribution {
              // The source reads before its row where a far edge mirrors past the near one; the near edge here.
              pixel: mirrored.clamp(0, from - 1) as usize,
              weight,
            }
          })
          .collect()
      })
      .collect()
  }

  /// One output pixel of `contributions` over the pixels `source` gives.
  fn get_pixel(contributions: &[Contribution], source: impl Fn(usize) -> [u8; 4]) -> [u8; 4] {
    let mut sums: [f32; 4] = [0.0; 4];

    for contribution in contributions {
      let pixel: [u8; 4] = source(contribution.pixel);

      for (sum, channel) in sums.iter_mut().zip(pixel) {
        *sum += contribution.weight * f32::from(channel);
      }
    }

    let clamp = |value: f32| ((value + 0.5).floor() as i32).clamp(0, 255) as u8;

    [clamp(sums[0]), clamp(sums[1]), clamp(sums[2]), clamp(sums[3] + 0.5)]
  }
}
