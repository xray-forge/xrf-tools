use xrf_math::glam::{Mat4, Vec2, Vec3, Vec4};

use crate::contract::viewport::render_rect::RenderRect;

/// A fixed grid of screen tiles bounding where a pass's inputs may draw, such as the distortion target or the water.
///
/// Each input marks the tiles its screen bounds touch, clipped at the near plane, or every tile where it cannot tell.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ScreenTiles {
  /// One bitmask per row, top row first, lowest bit the left column.
  rows: [u32; Self::ROWS as usize],
}

impl ScreenTiles {
  /// Grid columns and rows, near square tiles on a wide frame.
  pub(crate) const COLUMNS: u32 = 32;
  pub(crate) const ROWS: u32 = 18;

  const ALL_COLUMNS: u32 = u32::MAX >> (32 - Self::COLUMNS);
  /// Screen fraction a bound grows by against lens jitter and raster rounding, a few pixels at any size.
  const MARGIN: f32 = 0.002;

  pub(crate) fn new_whole() -> Self {
    Self {
      rows: [Self::ALL_COLUMNS; Self::ROWS as usize],
    }
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.rows.iter().all(|row| *row == 0)
  }

  /// Adds the tiles `other` marks.
  pub(crate) fn cover_tiles(&mut self, other: &Self) {
    for (row, other) in self.rows.iter_mut().zip(other.rows) {
      *row |= other;
    }
  }

  /// Marks the tiles the convex hull of `points` covers under the reversed-depth `view_projection`, clipped at the near
  /// plane; nothing where the hull lies wholly in front of it.
  pub(crate) fn cover_hull(&mut self, view_projection: Mat4, points: &[Vec3]) {
    let mut low: Vec2 = Vec2::splat(f32::MAX);
    let mut high: Vec2 = Vec2::splat(f32::MIN);
    let mut take = |clip: Vec4| {
      let ndc: Vec2 = clip.truncate().truncate() / clip.w;

      low = low.min(ndc);
      high = high.max(ndc);
    };
    // Reversed depth is one at the near plane, so a point is beyond it where `w - z >= 0`.
    let past_near = |clip: Vec4| clip.w - clip.z;
    let clips = points.iter().map(|point| view_projection * point.extend(1.0));

    // The clipped hull is bounded by the points beyond the plane and the plane crossings of every pair across it.
    for (index, a) in clips.clone().enumerate() {
      if past_near(a) >= 0.0 {
        take(a);
      }

      for b in clips.clone().skip(index + 1) {
        if (past_near(a) >= 0.0) != (past_near(b) >= 0.0) {
          take(a.lerp(b, past_near(a) / (past_near(a) - past_near(b))));
        }
      }
    }

    if low.x <= high.x {
      self.cover_ndc(low, high);
    }
  }

  /// Marks the tiles the bounding box of `sphere` (centre `xyz`, radius `w`) covers under `view_projection`.
  pub(crate) fn cover_sphere(&mut self, view_projection: Mat4, sphere: Vec4) {
    let (center, radius) = (sphere.truncate(), Vec3::splat(sphere.w));
    let corners: [Vec3; 8] = std::array::from_fn(|index| {
      let side: Vec3 = Vec3::new(
        if index & 1 == 0 { -1.0 } else { 1.0 },
        if index & 2 == 0 { -1.0 } else { 1.0 },
        if index & 4 == 0 { -1.0 } else { 1.0 },
      );

      center + side * radius
    });

    self.cover_hull(view_projection, &corners);
  }

  /// Marks the tiles an NDC rectangle grown by `MARGIN` touches; nothing for one off screen.
  fn cover_ndc(&mut self, low: Vec2, high: Vec2) {
    let (low, high) = (low - Self::MARGIN, high + Self::MARGIN);

    if high.x < -1.0 || high.y < -1.0 || low.x > 1.0 || low.y > 1.0 {
      return;
    }

    let grid: Vec2 = Vec2::new(Self::COLUMNS as f32, Self::ROWS as f32);
    // NDC `y` points up; rows go down.
    let tile = |ndc: Vec2| -> (u32, u32) {
      let at: Vec2 = ((Vec2::new(ndc.x, -ndc.y) * 0.5 + 0.5) * grid).clamp(Vec2::ZERO, grid - 1.0);

      (at.x as u32, at.y as u32)
    };
    let ((left, top), (right, bottom)) = (tile(Vec2::new(low.x, high.y)), tile(Vec2::new(high.x, low.y)));
    let columns: u32 = (Self::ALL_COLUMNS >> (Self::COLUMNS - 1 - (right - left))) << left;

    for row in &mut self.rows[top as usize..=bottom as usize] {
      *row |= columns;
    }
  }

  /// These tiles dilated by `columns` horizontally and `rows` vertically.
  pub(crate) fn grown(&self, columns: u32, rows: u32) -> Self {
    let mut across: [u32; Self::ROWS as usize] = self.rows;

    for row in &mut across {
      let marked: u32 = *row;

      for step in 1..=columns.min(Self::COLUMNS - 1) {
        *row |= (marked << step) | (marked >> step);
      }

      *row &= Self::ALL_COLUMNS;
    }

    let mut grown: Self = Self::default();

    for (index, row) in grown.rows.iter_mut().enumerate() {
      let first: usize = index.saturating_sub(rows as usize);
      let last: usize = (index + rows as usize).min(Self::ROWS as usize - 1);

      *row = across[first..=last].iter().fold(0, |row, other| row | other);
    }

    grown
  }

  /// The marked tiles as pixel rectangles of a `size` frame, merging identical runs in consecutive rows.
  pub(crate) fn get_rects(&self, (width, height): (u32, u32)) -> Vec<RenderRect> {
    let x_of = |column: u32| column * width / Self::COLUMNS;
    let y_of = |row: u32| row * height / Self::ROWS;
    let mut rects: Vec<RenderRect> = Vec::new();
    let mut open: Vec<(u32, u32, u32)> = Vec::new();

    for row in 0..=Self::ROWS {
      let marked: u32 = self.rows.get(row as usize).copied().unwrap_or(0);
      let runs: Vec<(u32, u32)> = Self::list_runs(marked);
      let (kept, closed): (Vec<_>, Vec<_>) = open
        .into_iter()
        .partition(|(left, right, _)| runs.contains(&(*left, *right)));

      for (left, right, top) in closed {
        rects.push(RenderRect {
          x: x_of(left).cast_signed(),
          y: y_of(top).cast_signed(),
          width: x_of(right) - x_of(left),
          height: y_of(row) - y_of(top),
        });
      }

      open = runs
        .into_iter()
        .map(|(left, right)| {
          kept
            .iter()
            .find(|(l, r, _)| (*l, *r) == (left, right))
            .copied()
            .unwrap_or((left, right, row))
        })
        .collect();
    }

    rects
  }

  /// A row's marked runs as half-open column ranges.
  fn list_runs(mut marked: u32) -> Vec<(u32, u32)> {
    let mut runs: Vec<(u32, u32)> = Vec::new();

    while marked != 0 {
      let left: u32 = marked.trailing_zeros();
      let length: u32 = (marked >> left).trailing_ones();

      runs.push((left, left + length));
      marked &= !((u32::MAX >> (32 - length)) << left);
    }

    runs
  }
}
