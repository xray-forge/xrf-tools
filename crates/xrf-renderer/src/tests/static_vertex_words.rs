use glam::Vec3;

use crate::scene::static_scene::static_vertex_words::StaticVertexWords;

/// A packed direction as `vertex.wgsl` reads it: the word's bytes as `unorm8x4`, then `.zyx * 2 - 1`.
fn unpack(word: u32) -> Vec3 {
  let byte = |shift: u32| -> f32 { ((word >> shift) & 0xFF) as f32 / 255.0 * 2.0 - 1.0 };

  Vec3::new(byte(16), byte(8), byte(0))
}

// A geometry missing its tangent frame is lit and bumped along a square frame, never a direction of nought.
#[test]
fn fills_a_missing_tangent_frame_with_the_three_axes() {
  for (word, axis) in [
    (StaticVertexWords::NEUTRAL_TANGENT, Vec3::X),
    (StaticVertexWords::NEUTRAL_BINORMAL, Vec3::Y),
    (StaticVertexWords::NEUTRAL_NORMAL, Vec3::Z),
  ] {
    assert_eq!(word, StaticVertexWords::pack_direction(axis.to_array()));
    assert!(
      unpack(word).normalize().abs_diff_eq(axis, 0.01),
      "{:?} unpacks off {axis}",
      unpack(word)
    );
  }
}
