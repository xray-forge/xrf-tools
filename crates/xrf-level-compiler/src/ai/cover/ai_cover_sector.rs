use xrf_math::Vector3d;

/// The eight horizontal sectors a cover node bins its rays in (`calcSphereSector`, `xrAI/compiler_cover.cpp:142`), and
/// the four covers each averaging four of them.
pub(crate) struct AiCoverSector;

impl AiCoverSector {
  /// The sector of `direction`, a ray's unit direction: its x and z normalised again on the ground, then split by which
  /// is longer and their signs. A vertical direction normalises to NaNs, every comparison fails, and it lands in
  /// sector 6, as in the engine.
  pub(crate) fn of(direction: &Vector3d) -> usize {
    let length: f32 = (direction.x * direction.x + direction.z * direction.z).sqrt();
    let (x, y): (f32, f32) = (direction.x / length, direction.z / length);

    if x.abs() > y.abs() {
      match (x < 0.0, y > 0.0) {
        (true, true) => 3,
        (true, false) => 4,
        (false, true) => 0,
        (false, false) => 7,
      }
    } else {
      match (x < 0.0, y > 0.0) {
        (true, true) => 2,
        (true, false) => 5,
        (false, true) => 1,
        (false, false) => 6,
      }
    }
  }

  /// The four covers, left, forward, right and back, from the eight sectors' passed shares: each the mean of four
  /// sectors, summed in the engine's order (`:293`), clamped to 0..1.
  pub(crate) fn get_covers(values: &[f32; 8]) -> [f32; 4] {
    let mean =
      |a: usize, b: usize, c: usize, d: usize| ((values[a] + values[b] + values[c] + values[d]) / 4.0).clamp(0.0, 1.0);

    [mean(2, 3, 4, 5), mean(0, 1, 2, 3), mean(6, 7, 0, 1), mean(4, 5, 6, 7)]
  }
}
