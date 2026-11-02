use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::contract::viewport::render_rect::RenderRect;

/// Where on the frame this frame's writers draw into the distortion target, as tiles of a fixed grid over the frame:
/// what the distortion copies the frame for and moves it in, nothing where no tile is marked. A writer marks the tiles
/// its screen bounds touch, clipped at the eye, or every tile where it cannot say.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DistortionTiles {
  /// A row's marked tiles a bit each, the lowest bit the left column, the first row the top.
  rows: [u32; Self::ROWS as usize],
}

impl DistortionTiles {
  /// Columns and rows the frame is cut into, near square at a wide frame.
  pub const COLUMNS: u32 = 32;
  pub const ROWS: u32 = 18;

  /// A row with every column marked.
  const ALL_COLUMNS: u32 = u32::MAX >> (32 - Self::COLUMNS);
  /// Clip-space depth (`w`) a bound is clipped at: just before the eye, the near plane standing beyond it.
  const EYE_W: f32 = 1e-4;
  /// The screen share a bound is grown by, a few pixels at any size, against the jittered lens and the raster's rounding.
  const MARGIN: f32 = 0.002;

  /// Every tile.
  pub fn new_whole() -> Self {
    Self {
      rows: [Self::ALL_COLUMNS; Self::ROWS as usize],
    }
  }

  /// Whether no tile is marked.
  pub fn is_empty(&self) -> bool {
    self.rows.iter().all(|row| *row == 0)
  }

  /// Marks every tile another marks too.
  pub fn cover_tiles(&mut self, other: &Self) {
    for (row, other) in self.rows.iter_mut().zip(other.rows) {
      *row |= other;
    }
  }

  /// Marks the tiles the convex hull of `points` touches on the screen of `view_projection`, clipped at the eye:
  /// none where it stands wholly behind it.
  pub fn cover_hull(&mut self, view_projection: Mat4, points: &[Vec3]) {
    let mut low: Vec2 = Vec2::splat(f32::MAX);
    let mut high: Vec2 = Vec2::splat(f32::MIN);
    let mut take = |clip: Vec4| {
      let ndc: Vec2 = clip.truncate().truncate() / clip.w;

      low = low.min(ndc);
      high = high.max(ndc);
    };
    let clips = points.iter().map(|point| view_projection * point.extend(1.0));

    // The hull's part before the eye lies within the points before it and where any two of them across it cross.
    for (index, a) in clips.clone().enumerate() {
      if a.w >= Self::EYE_W {
        take(a);
      }

      for b in clips.clone().skip(index + 1) {
        if (a.w >= Self::EYE_W) != (b.w >= Self::EYE_W) {
          take(a.lerp(b, (Self::EYE_W - a.w) / (b.w - a.w)));
        }
      }
    }

    if low.x <= high.x {
      self.cover_ndc(low, high);
    }
  }

  /// Marks the tiles a sphere (`xyz` its centre, `w` its radius) touches on the screen of `view_projection`, by the box
  /// about it.
  pub fn cover_sphere(&mut self, view_projection: Mat4, sphere: Vec4) {
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

  /// Marks the tiles a rectangle of normalized device coordinates touches, grown by the margin; none off the screen.
  fn cover_ndc(&mut self, low: Vec2, high: Vec2) {
    let (low, high) = (low - Self::MARGIN, high + Self::MARGIN);

    if high.x < -1.0 || high.y < -1.0 || low.x > 1.0 || low.y > 1.0 {
      return;
    }

    let grid: Vec2 = Vec2::new(Self::COLUMNS as f32, Self::ROWS as f32);
    // Device `y` points up, rows go down the frame.
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

  /// These tiles and those within `columns` across and `rows` down of a marked one.
  pub fn grown(&self, columns: u32, rows: u32) -> Self {
    let mut across: [u32; Self::ROWS as usize] = self.rows;

    for row in &mut across {
      let marked: u32 = *row;

      for step in 1..=columns.min(Self::COLUMNS - 1) {
        *row |= marked << step | marked >> step;
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

  /// The marked tiles as pixel rectangles of a frame of `size`, each run of marked tiles along a row joined with the
  /// same run in the rows under it.
  pub fn get_rects(&self, (width, height): (u32, u32)) -> Vec<RenderRect> {
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
          x: x_of(left) as i32,
          y: y_of(top) as i32,
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

  /// A row's runs of marked tiles, each its first column and the one past its last.
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
