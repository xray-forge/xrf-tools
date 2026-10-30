use std::ops::Range;

use glam::Vec3;

/// One helped object as a scene keeps it: what names it, how it is drawn, where it stands, its wires among the scene's,
/// and the spawned model it marks.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelHelperObject {
  pub id: u32,
  pub kind: u32,
  pub color: [f32; 3],
  pub draw_distance: f32,
  /// Where it stands, renderer space: what a pick of its wire names as the point met.
  pub place: Vec3,
  /// Its wires, by index among the scene's segments.
  pub segments: Range<usize>,
  /// The spawned object whose drawn model it marks, by its index among the level's spawned objects.
  pub model: Option<u32>,
}
