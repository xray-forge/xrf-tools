use std::ops::RangeInclusive;

use xrf_math::Vector3d;

/// Unit vectors as the three bytes a bump texture stores them in, `vpack` (`xray/trunk/xrDXT/NormalMapGen.cpp`).
///
/// A byte triple decodes to a vector that is rarely unit length, so rounding each component on its own leaves the
/// decoded direction drifted, the three roundings not cancelling. The SDK searches a small cube around that first
/// guess and keeps the candidate whose decoded direction points most nearly the same way, refusing any whose decoded
/// length has strayed too far from one, which is what keeps a renormalizing shader from having to correct it.
pub(crate) struct BumpVector;

impl BumpVector {
  /// How far either side of the naive packing the search looks, in byte steps.
  const SEARCH_RADIUS: u8 = 2;

  /// How far a candidate's decoded length may sit from one before it is refused outright.
  const LENGTH_TOLERANCE: f32 = 0.03;

  /// A direction in the range a byte covers, `[-1, 1]` mapped onto `0..=255`, `fpack`.
  pub(crate) fn pack_component(value: f32) -> u8 {
    (((value + 1.0) * 0.5 * 255.0 + 0.5).floor() as i32).clamp(0, 255) as u8
  }

  /// The direction a byte triple decodes to, before any normalization, `vunpack`.
  pub(crate) fn unpack(bytes: [u8; 3]) -> Vector3d {
    let [x, y, z] = bytes.map(|byte| (f32::from(byte) / 255.0 - 0.5) * 2.0);

    Vector3d::new(x, y, z)
  }

  /// The byte triple that decodes closest to `vector`, which is not the one that rounds closest to it.
  pub(crate) fn pack(vector: &Vector3d) -> [u8; 3] {
    let direction: Vector3d = Self::normalize(vector);
    let guess: [u8; 3] = [direction.x, direction.y, direction.z].map(Self::pack_component);
    let mut best: [u8; 3] = guess;
    let mut best_error: f32 = f32::MAX;

    for x in Self::search_range(guess[0]) {
      for y in Self::search_range(guess[1]) {
        for z in Self::search_range(guess[2]) {
          let candidate: Vector3d = Self::unpack([x, y, z]);
          let length: f32 = candidate.length();

          if (length - 1.0).abs() > Self::LENGTH_TOLERANCE {
            continue;
          }

          // One minus the cosine between the two, so nought is a candidate pointing exactly where it was asked to.
          let error: f32 = (direction.dot(&(&candidate / length)) - 1.0).abs();

          if error < best_error {
            best_error = error;
            best = [x, y, z];
          }
        }
      }
    }

    best
  }

  /// `Fvector::normalize`, which scales by the square root of the reciprocal rather than dividing by the length; a
  /// vector with no length is kept, where the SDK asserts.
  fn normalize(vector: &Vector3d) -> Vector3d {
    match vector.length_squared() {
      0.0 => vector.clone(),
      squared => vector * (1.0 / squared).sqrt(),
    }
  }

  /// The bytes the search walks around one component of the first guess.
  fn search_range(center: u8) -> RangeInclusive<u8> {
    center.saturating_sub(Self::SEARCH_RADIUS)..=center.saturating_add(Self::SEARCH_RADIUS)
  }
}

#[cfg(test)]
mod tests {
  use xrf_math::Vector3d;

  use super::BumpVector;

  #[test]
  fn a_component_covers_the_byte_range_it_is_given() {
    // The two ends and the middle of `[-1, 1]`, which is where a flat surface's normal sits.
    assert_eq!(BumpVector::pack_component(-1.0), 0);
    assert_eq!(BumpVector::pack_component(0.0), 128);
    assert_eq!(BumpVector::pack_component(1.0), 255);

    // Past either end is clamped rather than wrapped, since a normal arriving unnormalized is not a reason to write
    // a byte that decodes to the opposite direction.
    assert_eq!(BumpVector::pack_component(-4.0), 0);
    assert_eq!(BumpVector::pack_component(4.0), 255);
  }

  #[test]
  fn packing_and_unpacking_round_trip_through_the_byte() {
    for byte in 0..=u8::MAX {
      let component: f32 = BumpVector::unpack([byte, 0, 0]).x;

      assert_eq!(
        BumpVector::pack_component(component),
        byte,
        "byte {byte} did not survive"
      );
    }
  }

  #[test]
  fn a_flat_normal_packs_to_the_value_the_shipped_textures_use() {
    // The surface that bends nowhere. Rounding alone would answer 128, but 127 and 128 decode to directions equally
    // far from straight up - one leaning a thousandth each way - and the search keeps the first it meets.
    //
    // 127 is what the SDK wrote: it is the most common `normal.x` in every shipped bump texture sampled from
    // `gamedata` and `gamedata-anomaly`, with 126 and 128 either side of it.
    assert_eq!(BumpVector::pack(&Vector3d::Z), [127, 127, 255]);
  }

  #[test]
  fn the_search_keeps_the_direction_rather_than_the_rounding() {
    // A direction whose components do not land on byte boundaries. The search is allowed to move each byte by two, so
    // its answer stays near the naive rounding while decoding closer to where it was pointed.
    let direction: Vector3d = Vector3d::new(0.267_261, 0.534_522, 0.801_784);
    let packed: [u8; 3] = BumpVector::pack(&direction);
    let guess: [u8; 3] = [direction.x, direction.y, direction.z].map(BumpVector::pack_component);

    for channel in 0..3 {
      assert!(
        packed[channel].abs_diff(guess[channel]) <= 2,
        "channel {channel} moved further than the search allows"
      );
    }

    // What the search is for: the decoded direction is closer to the one asked for than the naive rounding is.
    let cosine = |bytes: [u8; 3]| {
      let decoded: Vector3d = BumpVector::unpack(bytes);

      direction.dot(&decoded) / decoded.length()
    };

    assert!(
      cosine(packed) >= cosine(guess),
      "the search answered a worse direction than rounding did"
    );
  }
}
