use xrf_math::Vector2d;

use crate::chart::light_chart::LightChart;

/// The texels a chart takes on a page, border and all, as xrLC places it (`_rect_place`, marker at least `254 -
/// BORDER`): those its faces light, two rings of neighbours round them, within its rectangle. Kept as each row's span
/// from its first texel to its last, upright and crosswise, which the packer places by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightChartMask {
  /// Its rectangle's texels a side, borders included.
  pub width: u32,
  pub height: u32,
  /// Each row's first and last texel taken, none for a row it leaves empty.
  pub rows: Vec<Option<(u32, u32)>>,
  /// Each column's, the rows of the mask laid crosswise.
  pub columns: Vec<Option<(u32, u32)>>,
}

impl LightChartMask {
  /// `Jitter_Select`'s offsets, in half texels, for 1, 4 and 9 samples a texel.
  const JITTER_1: [[f32; 2]; 1] = [[0.0, 0.0]];
  const JITTER_4: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
  const JITTER_9: [[f32; 2]; 9] = [
    [-1.0, -1.0],
    [0.0, -1.0],
    [1.0, -1.0],
    [-1.0, 0.0],
    [0.0, 0.0],
    [1.0, 0.0],
    [-1.0, 1.0],
    [0.0, 1.0],
    [1.0, 1.0],
  ];

  /// The mask of `chart` lit with `jitter_samples` a texel. A texel is lit where one of its samples falls on a face,
  /// its corners mapped into the chart's own texels (`CDeflector::L_Direct`), or where a walk along a face's edge in
  /// steps of an eighth of a texel lands in it (`L_Direct_Edge`). A chart one texel across is its whole two-texel strip
  /// (`CDeflector::Light`); any other its lit texels and the two rings round them `ApplyBorders` fills, within its
  /// rectangle a border wider each side.
  pub fn of(chart: &LightChart, jitter_samples: u32) -> Self {
    let (width, height) = (chart.width, chart.height);
    let border: u32 = LightChart::BORDER;

    if width == 1 || height == 1 {
      let (mask_width, mask_height) = if width == 1 {
        (2 * border, height + 2 * border)
      } else {
        (width + 2 * border, 2 * border)
      };

      return Self::from_texels(
        mask_width,
        mask_height,
        &vec![true; (mask_width * mask_height) as usize],
      );
    }

    let lit: Vec<bool> = Self::get_lit(chart, jitter_samples);
    let (mask_width, mask_height) = (width + 2 * border, height + 2 * border);
    let mut texels: Vec<bool> = vec![false; (mask_width * mask_height) as usize];

    for y in 0..height {
      for x in 0..width {
        texels[((y + border) * mask_width + x + border) as usize] = lit[(y * width + x) as usize];
      }
    }

    for _ in 0..2 {
      texels = Self::grow(&texels, mask_width, mask_height);
    }

    Self::from_texels(mask_width, mask_height, &texels)
  }

  /// The texels of the rectangle it takes, `Area`.
  pub fn get_area(&self) -> u32 {
    self.width * self.height
  }

  /// Each row's span, upright or crosswise.
  pub fn get_rows(&self, is_rotated: bool) -> &[Option<(u32, u32)>] {
    if is_rotated { &self.columns } else { &self.rows }
  }

  /// The chart's lit texels, row by row over its own `width` by `height`.
  fn get_lit(chart: &LightChart, jitter_samples: u32) -> Vec<bool> {
    let (width, height) = (chart.width, chart.height);
    let mut lit: Vec<bool> = vec![false; (width * height) as usize];
    let triangles: Vec<[Vector2d; 3]> = chart.remap([0, 0], [width, height], [width, height], false);
    let jitter: &[[f32; 2]] = match jitter_samples {
      1 => &Self::JITTER_1,
      9 => &Self::JITTER_9,
      _ => &Self::JITTER_4,
    };
    let (dimension_x, dimension_y) = (width as f32, height as f32);
    let (half_x, half_y) = (0.5 / dimension_x, 0.5 / dimension_y);
    let (spread_x, spread_y) = (0.4999 / dimension_x, 0.4999 / dimension_y);

    for triangle in &triangles {
      let low_x: f32 = triangle.iter().map(|corner| corner.x).fold(f32::INFINITY, f32::min);
      let high_x: f32 = triangle.iter().map(|corner| corner.x).fold(f32::NEG_INFINITY, f32::max);
      let low_y: f32 = triangle.iter().map(|corner| corner.y).fold(f32::INFINITY, f32::min);
      let high_y: f32 = triangle.iter().map(|corner| corner.y).fold(f32::NEG_INFINITY, f32::max);
      let range = |low: f32, high: f32, dimension: f32, side: u32| {
        let first: i64 = ((low * dimension).floor() as i64 - 1).max(0);
        let last: i64 = ((high * dimension).ceil() as i64 + 1).min(i64::from(side) - 1);

        first..=last
      };

      for y in range(low_y, high_y, dimension_y, height) {
        for x in range(low_x, high_x, dimension_x, width) {
          let index: usize = (y as u32 * width + x as u32) as usize;

          if lit[index] {
            continue;
          }

          lit[index] = jitter.iter().any(|[jitter_x, jitter_y]| {
            let point: Vector2d = Vector2d::new(
              x as f32 / dimension_x + half_x + jitter_x * spread_x,
              y as f32 / dimension_y + half_y + jitter_y * spread_y,
            );

            Self::is_inside(triangle, &point)
          });
        }
      }
    }

    let texel_size: f32 = (1.0 / width.max(height) as f32) / 8.0;

    for triangle in &triangles {
      for (first, second) in [(0, 1), (1, 2), (2, 0)] {
        let (start, end) = (triangle[first], triangle[second]);
        let (size_x, size_y) = (end.x - start.x, end.y - start.y);
        let steps: i64 = ((size_x.abs() / texel_size).ceil() as i64).max((size_y.abs() / texel_size).ceil() as i64);

        if steps <= 0 {
          continue;
        }

        for step in 0..=steps {
          let time: f32 = step as f32 / steps as f32;
          let x: i64 = ((size_x * time + start.x) * dimension_x).floor() as i64;
          let y: i64 = ((size_y * time + start.y) * dimension_y).floor() as i64;

          if (0..i64::from(width)).contains(&x) && (0..i64::from(height)).contains(&y) {
            lit[(y as u32 * width + x as u32) as usize] = true;
          }
        }
      }
    }

    lit
  }

  /// `_TCF::isInside`: whether `point`'s barycentric coordinates in `triangle` each lie between 0 and 1.
  fn is_inside(triangle: &[Vector2d; 3], point: &Vector2d) -> bool {
    let [first, second, third] = triangle;
    let (v02_x, v02_y) = (first.x - third.x, first.y - third.y);
    let (v12_x, v12_y) = (second.x - third.x, second.y - third.y);
    let (pv2_x, pv2_y) = (point.x - third.x, point.y - third.y);
    let m00: f32 = v02_x * v02_x + v02_y * v02_y;
    let m01: f32 = v02_x * v12_x + v02_y * v12_y;
    let m11: f32 = v12_x * v12_x + v12_y * v12_y;
    let r0: f32 = v02_x * pv2_x + v02_y * pv2_y;
    let r1: f32 = v12_x * pv2_x + v12_y * pv2_y;
    let determinant: f32 = m00 * m11 - m01 * m01;
    let u: f32 = (m11 * r0 - m01 * r1) / determinant;
    let v: f32 = (m00 * r1 - m01 * r0) / determinant;
    let w: f32 = 1.0 - u - v;

    (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) && (0.0..=1.0).contains(&w)
  }

  /// `ApplyBorders` once: every texel taken, and every one beside or across a corner from one.
  fn grow(texels: &[bool], width: u32, height: u32) -> Vec<bool> {
    let (width, height) = (i64::from(width), i64::from(height));

    (0..height)
      .flat_map(|y| (0..width).map(move |x| (x, y)))
      .map(|(x, y)| {
        (-1..=1).any(|dy| {
          (-1..=1).any(|dx| {
            let (nx, ny) = (x + dx, y + dy);

            (0..width).contains(&nx) && (0..height).contains(&ny) && texels[(ny * width + nx) as usize]
          })
        })
      })
      .collect()
  }

  /// The mask of `texels`, row by row over `width` by `height`, its spans read off.
  fn from_texels(width: u32, height: u32, texels: &[bool]) -> Self {
    let span = |taken: &mut dyn Iterator<Item = (u32, bool)>| {
      taken
        .filter(|(_, is_taken)| *is_taken)
        .fold(None, |span: Option<(u32, u32)>, (at, _)| {
          Some(span.map_or((at, at), |(first, _)| (first, at)))
        })
    };

    Self {
      width,
      height,
      rows: (0..height)
        .map(|y| span(&mut (0..width).map(|x| (x, texels[(y * width + x) as usize]))))
        .collect(),
      columns: (0..width)
        .map(|x| span(&mut (0..height).map(|y| (y, texels[(y * width + x) as usize]))))
        .collect(),
    }
  }
}
