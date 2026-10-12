use glam::{Mat4, Vec3};

use crate::contract::render_overlay::RenderOverlay;

/// A box's twelve edges as lines, its corners where `transform` stands them, in one colour seen through what is in front.
pub fn to_box_lines(transform: Mat4, [min, max]: [Vec3; 2], color: [f32; 3]) -> RenderOverlay {
  let corner = |index: usize| -> Vec3 {
    transform.transform_point3(Vec3::new(
      if index & 1 == 0 { min.x } else { max.x },
      if index & 2 == 0 { min.y } else { max.y },
      if index & 4 == 0 { min.z } else { max.z },
    ))
  };
  let edges: [(usize, usize); 12] = [
    (0, 1),
    (2, 3),
    (4, 5),
    (6, 7),
    (0, 2),
    (1, 3),
    (4, 6),
    (5, 7),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
  ];
  let positions: Vec<f32> = edges
    .iter()
    .flat_map(|(from, to)| [corner(*from), corner(*to)])
    .flat_map(|point| point.to_array())
    .collect();

  RenderOverlay::Lines {
    colors: color.repeat(positions.len() / 3),
    positions,
    is_depth_tested: false,
  }
}
