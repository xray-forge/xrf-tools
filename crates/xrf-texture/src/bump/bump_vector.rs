//! Unit vectors as the three bytes a bump texture stores them in.
//!
//! Its own module because the packing is not obvious and is not a texture concern: a byte triple decodes to a vector
//! that is rarely unit length, and the SDK spends a local search picking the triple that decodes closest to the
//! direction it was given (`vpack`, `xray/trunk/xrDXT/NormalMapGen.cpp`).

/// How far either side of the naive packing the search looks, in byte steps.
const SEARCH_RADIUS: i32 = 2;

/// How far a candidate's decoded length may sit from one before it is refused outright.
const LENGTH_TOLERANCE: f32 = 0.03;

/// A direction in the range a byte covers, `[-1, 1]` mapped onto `0..=255`.
pub(crate) fn pack_component(value: f32) -> u8 {
  (((value + 1.0) * 0.5 * 255.0 + 0.5).floor() as i32).clamp(0, 255) as u8
}

/// The direction a byte triple decodes to, before any normalization.
pub(crate) fn unpack_vector(x: u8, y: u8, z: u8) -> [f32; 3] {
  [
    (f32::from(x) / 255.0 - 0.5) * 2.0,
    (f32::from(y) / 255.0 - 0.5) * 2.0,
    (f32::from(z) / 255.0 - 0.5) * 2.0,
  ]
}

/// The byte triple that decodes closest to `vector`, which is not the one that rounds closest to it.
///
/// Rounding each component on its own gives a triple whose decoded direction has drifted, because the three roundings
/// do not cancel. The SDK searches a small cube around that first guess and keeps the candidate whose decoded
/// direction points most nearly the same way, refusing any whose decoded length has strayed too far from one - which
/// is what keeps a renormalizing shader from having to correct it.
pub(crate) fn pack_vector(vector: [f32; 3]) -> [u8; 3] {
  let direction: [f32; 3] = normalize(vector);
  let guess: [u8; 3] = [
    pack_component(direction[0]),
    pack_component(direction[1]),
    pack_component(direction[2]),
  ];

  let mut best: [u8; 3] = guess;
  let mut best_error: f32 = f32::MAX;

  for x in search_range(guess[0]) {
    for y in search_range(guess[1]) {
      for z in search_range(guess[2]) {
        let candidate: [f32; 3] = unpack_vector(x, y, z);
        let length: f32 = magnitude(candidate);

        if (length - 1.0).abs() > LENGTH_TOLERANCE {
          continue;
        }

        // One minus the cosine between the two, so nought is a candidate pointing exactly where it was asked to.
        let error: f32 = (dot(direction, scale(candidate, 1.0 / length)) - 1.0).abs();

        if error < best_error {
          best_error = error;
          best = [x, y, z];
        }
      }
    }
  }

  best
}

/// The bytes the search walks around one component of the first guess.
fn search_range(center: u8) -> std::ops::RangeInclusive<u8> {
  let center: i32 = i32::from(center);

  (center - SEARCH_RADIUS).max(0) as u8..=(center + SEARCH_RADIUS).min(255) as u8
}

fn magnitude(vector: [f32; 3]) -> f32 {
  dot(vector, vector).sqrt()
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
  left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn scale(vector: [f32; 3], factor: f32) -> [f32; 3] {
  [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

/// A vector of length one, or the vector itself when it has no length to divide by.
fn normalize(vector: [f32; 3]) -> [f32; 3] {
  match magnitude(vector) {
    0.0 => vector,
    length => scale(vector, 1.0 / length),
  }
}

#[cfg(test)]
mod tests {
  use super::{pack_component, pack_vector, unpack_vector};

  #[test]
  fn a_component_covers_the_byte_range_it_is_given() {
    // The two ends and the middle of `[-1, 1]`, which is where a flat surface's normal sits.
    assert_eq!(pack_component(-1.0), 0);
    assert_eq!(pack_component(0.0), 128);
    assert_eq!(pack_component(1.0), 255);

    // Past either end is clamped rather than wrapped, since a normal arriving unnormalized is not a reason to write
    // a byte that decodes to the opposite direction.
    assert_eq!(pack_component(-4.0), 0);
    assert_eq!(pack_component(4.0), 255);
  }

  #[test]
  fn packing_and_unpacking_round_trip_through_the_byte() {
    for byte in 0..=u8::MAX {
      let [component, _, _] = unpack_vector(byte, 0, 0);

      assert_eq!(pack_component(component), byte, "byte {byte} did not survive");
    }
  }

  #[test]
  fn a_flat_normal_packs_to_the_value_the_shipped_textures_use() {
    // The surface that bends nowhere. Rounding alone would answer 128, but 127 and 128 decode to directions equally
    // far from straight up - one leaning a thousandth each way - and the search keeps the first it meets.
    //
    // 127 is what the SDK wrote: it is the most common `normal.x` in every shipped bump texture sampled from
    // `gamedata` and `gamedata-anomaly`, with 126 and 128 either side of it.
    assert_eq!(pack_vector([0.0, 0.0, 1.0]), [127, 127, 255]);
  }

  #[test]
  fn the_search_keeps_the_direction_rather_than_the_rounding() {
    // A direction whose components do not land on byte boundaries. The search is allowed to move each byte by two, so
    // its answer stays near the naive rounding while decoding closer to where it was pointed.
    let direction: [f32; 3] = [0.267_261, 0.534_522, 0.801_784];
    let packed: [u8; 3] = pack_vector(direction);
    let guess: [u8; 3] = [
      pack_component(direction[0]),
      pack_component(direction[1]),
      pack_component(direction[2]),
    ];

    for channel in 0..3 {
      assert!(
        packed[channel].abs_diff(guess[channel]) <= 2,
        "channel {channel} moved further than the search allows"
      );
    }

    // What the search is for: the decoded direction is closer to the one asked for than the naive rounding is.
    let decoded: [f32; 3] = unpack_vector(packed[0], packed[1], packed[2]);
    let naive: [f32; 3] = unpack_vector(guess[0], guess[1], guess[2]);
    let cosine = |vector: [f32; 3]| {
      let length: f32 = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();

      (direction[0] * vector[0] + direction[1] * vector[1] + direction[2] * vector[2]) / length
    };

    assert!(
      cosine(decoded) >= cosine(naive),
      "the search answered a worse direction than rounding did"
    );
  }
}
