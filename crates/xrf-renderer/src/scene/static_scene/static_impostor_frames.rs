use glam::{UVec2, Vec2, Vec3};

/// The views a baked impostor is drawn from, as `common/impostor_frames.wgsl` lays them out and reads them back: the
/// upper hemisphere hemi-octahedrally on a square of `FRAMES` × `FRAMES` frames, each frame's direction at a point of
/// the grid. Directions are renderer space's, `+y` up, each from the tree towards its camera.
pub struct StaticImpostorFrames;

impl StaticImpostorFrames {
  /// Frames each way, `IMPOSTOR_FRAMES`: Epic's for far foliage.
  pub const FRAMES: u32 = 12;

  /// Where a direction of the upper hemisphere lands on the square, `[0, 1]` each way.
  pub fn encode(direction: Vec3) -> Vec2 {
    let upper: Vec3 = Vec3::new(direction.x, direction.y.max(0.0), direction.z);
    let folded: Vec2 = Vec2::new(upper.x, upper.z) / (upper.x.abs() + upper.y + upper.z.abs()).max(1e-6);

    Vec2::new(folded.x + folded.y, folded.x - folded.y) * 0.5 + 0.5
  }

  /// The direction a point of the square stands for.
  pub fn decode(uv: Vec2) -> Vec3 {
    let grid: Vec2 = uv * 2.0 - 1.0;
    let folded: Vec2 = Vec2::new(grid.x + grid.y, grid.x - grid.y) * 0.5;

    Vec3::new(folded.x, 1.0 - folded.x.abs() - folded.y.abs(), folded.y).normalize()
  }

  /// The direction a frame is baked from.
  pub fn get_direction(frame: UVec2) -> Vec3 {
    Self::decode(frame.as_vec2() / (Self::FRAMES - 1) as f32)
  }

  /// A frame's right and up as its camera sees it, looking back along its direction.
  pub fn get_basis(direction: Vec3) -> (Vec3, Vec3) {
    let hint: Vec3 = if direction.y.abs() > 0.999 {
      Vec3::NEG_Z
    } else {
      Vec3::Y
    };
    let right: Vec3 = hint.cross(direction).normalize();

    (right, direction.cross(right))
  }

  /// Every frame, row by row.
  pub fn list() -> impl Iterator<Item = UVec2> {
    (0..Self::FRAMES).flat_map(|y| (0..Self::FRAMES).map(move |x| UVec2::new(x, y)))
  }
}
