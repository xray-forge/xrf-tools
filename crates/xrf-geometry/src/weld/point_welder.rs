use std::collections::HashMap;

use xrf_math::{EPS_L, Vector3d};

/// Points welded within [`EPS_L`] on every axis, as the editor's `VCPacked::add_vert` welds a part's points: a point
/// takes the index of one already held near it, else is held as a new one.
///
/// Held points are filed in cells [`EPS_L`] wide; a point looks through its own cell and the 26 around it, in `x`, `y`,
/// `z` order from the lowest, and takes the first held point it finds within reach there. Where several held points
/// stand within reach, that is the first in this scan, not necessarily the first held.
#[derive(Clone, Debug, Default)]
pub struct PointWelder {
  points: Vec<Vector3d>,
  cells: HashMap<[i64; 3], Vec<u32>>,
}

impl PointWelder {
  /// The index `point` welds to, holding it as a new point where none held is within reach.
  pub fn weld(&mut self, point: &Vector3d) -> u32 {
    let cell = |value: f32| (f64::from(value) / f64::from(EPS_L)).floor() as i64;
    let home: [i64; 3] = [cell(point.x), cell(point.y), cell(point.z)];

    for x in -1..=1 {
      for y in -1..=1 {
        for z in -1..=1 {
          let Some(candidates) = self.cells.get(&[home[0] + x, home[1] + y, home[2] + z]) else {
            continue;
          };

          if let Some(found) = candidates.iter().copied().find(|candidate| {
            let held: &Vector3d = &self.points[*candidate as usize];

            (held.x - point.x).abs() <= EPS_L && (held.y - point.y).abs() <= EPS_L && (held.z - point.z).abs() <= EPS_L
          }) {
            return found;
          }
        }
      }
    }

    let index: u32 = u32::try_from(self.points.len()).unwrap_or(u32::MAX);

    self.points.push(*point);
    self.cells.entry(home).or_default().push(index);

    index
  }

  /// The points held, by index.
  pub fn get_points(&self) -> &[Vector3d] {
    &self.points
  }
}
