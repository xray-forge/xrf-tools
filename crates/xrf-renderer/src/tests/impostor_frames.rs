//! The baked impostors' frames (`common/impostor_frames`), run on the GPU and held to their Rust mirror
//! (`StaticImpostorFrames`), which the bake's cameras are placed by.

use glam::{UVec2, Vec2, Vec3, Vec4};

use crate::scene::static_scene::static_impostor_frames::StaticImpostorFrames;
use crate::tests::wgsl_probe::run_wgsl_probe;

fn run(answer: &str, cases: &[(Vec4, Vec4)]) -> Option<Vec<Vec4>> {
  run_wgsl_probe("common/impostor_frames", answer, cases)
}

/// Directions over the upper hemisphere, straight up, along the horizon, and between.
fn directions() -> Vec<Vec3> {
  vec![
    Vec3::Y,
    Vec3::X,
    Vec3::NEG_Z,
    Vec3::new(0.3, 0.9, -0.2).normalize(),
    Vec3::new(-0.7, 0.1, 0.6).normalize(),
    Vec3::new(0.05, 0.5, 0.95).normalize(),
    Vec3::new(-0.4, 0.8, -0.4).normalize(),
  ]
}

#[test]
fn a_direction_lands_on_the_square_and_comes_back_as_itself() {
  let cases: Vec<(Vec4, Vec4)> = directions()
    .into_iter()
    .map(|it| (it.extend(0.0), Vec4::ZERO))
    .collect();
  let Some(answered) = run(
    "vec4<f32>(hemi_octahedral_decode(hemi_octahedral_encode(probe.first.xyz)), \
     f32(IMPOSTOR_FRAMES))",
    &cases,
  ) else {
    return;
  };

  for ((direction, _), found) in cases.iter().zip(answered) {
    assert!(
      found.truncate().distance(direction.truncate()) < 1e-5,
      "{direction} came back as {found}"
    );
    assert_eq!(
      found.w,
      StaticImpostorFrames::FRAMES as f32,
      "the WGSL frames are the Rust's"
    );
    assert!(
      StaticImpostorFrames::decode(StaticImpostorFrames::encode(direction.truncate())).distance(direction.truncate())
        < 1e-5,
      "and in Rust"
    );
  }
}

#[test]
fn a_frames_direction_and_basis_are_the_same_on_both_sides() {
  let frames: Vec<UVec2> = StaticImpostorFrames::list().collect();
  let cases: Vec<(Vec4, Vec4)> = frames
    .iter()
    .map(|frame| (Vec4::new(frame.x as f32, frame.y as f32, 0.0, 0.0), Vec4::ZERO))
    .collect();
  let (Some(directions), Some(rights)) = (
    run(
      "vec4<f32>(impostor_frame_direction(vec2<u32>(probe.first.xy)), 0.0)",
      &cases,
    ),
    run(
      "vec4<f32>(impostor_frame_basis(impostor_frame_direction(vec2<u32>(probe.first.xy)))[0], 0.0)",
      &cases,
    ),
  ) else {
    return;
  };

  assert_eq!(frames.len(), 144);

  for ((frame, direction), right) in frames.iter().zip(directions).zip(rights) {
    let expected: Vec3 = StaticImpostorFrames::get_direction(*frame);
    let (expected_right, up): (Vec3, Vec3) = StaticImpostorFrames::get_basis(expected);

    assert!(
      direction.truncate().distance(expected) < 1e-5,
      "frame {frame}: {direction} against {expected}"
    );
    assert!(
      right.truncate().distance(expected_right) < 1e-5,
      "frame {frame}'s right"
    );
    assert!(expected.y >= -1e-6, "frame {frame} looks from below");
    assert!(
      expected_right.dot(expected).abs() < 1e-5 && up.dot(expected).abs() < 1e-5 && (up.length() - 1.0).abs() < 1e-5,
      "frame {frame}'s basis is square to its direction"
    );
  }

  assert!(
    StaticImpostorFrames::get_direction(UVec2::splat(5)).distance(Vec3::Y) > 1e-3,
    "no frame sits at the top"
  );
  assert!(
    StaticImpostorFrames::get_direction(UVec2::ZERO).y.abs() < 1e-6,
    "the corner frame looks along the horizon"
  );
}

#[test]
fn a_direction_is_read_from_the_three_frames_around_it() {
  let cases: Vec<(Vec4, Vec4)> = directions()
    .into_iter()
    .chain([StaticImpostorFrames::get_direction(UVec2::new(4, 7))])
    .map(|it| (it.extend(0.0), Vec4::ZERO))
    .collect();
  // The weights, then where the three frames weighed by them stand on the grid.
  let (Some(weights), Some(blended)) = (
    run("vec4<f32>(impostor_frames(probe.first.xyz).weights, 0.0)", &cases),
    run(
      "vec4<f32>(vec2<f32>(impostor_frames(probe.first.xyz).first) * impostor_frames(probe.first.xyz).weights.x + \
       vec2<f32>(impostor_frames(probe.first.xyz).second) * impostor_frames(probe.first.xyz).weights.y + \
       vec2<f32>(impostor_frames(probe.first.xyz).third) * impostor_frames(probe.first.xyz).weights.z, 0.0, 0.0)",
      &cases,
    ),
  ) else {
    return;
  };

  for (((direction, _), weight), blended) in cases.iter().zip(&weights).zip(&blended) {
    let at: Vec2 = StaticImpostorFrames::encode(direction.truncate()) * (StaticImpostorFrames::FRAMES - 1) as f32;

    assert!((weight.x + weight.y + weight.z - 1.0).abs() < 1e-5, "{weight}");
    assert!(weight.min_element() >= -1e-5, "{weight}");
    // Weights that interpolate the frames' places back to the direction's are its triangle's barycentric ones.
    assert!(
      Vec2::new(blended.x, blended.y).distance(at) < 1e-4,
      "{direction}: {blended} against {at}"
    );
  }

  // A frame's own direction reads that frame alone.
  let own: usize = cases.len() - 1;

  assert!((weights[own].max_element() - 1.0).abs() < 1e-4, "{}", weights[own]);
  assert!(Vec2::new(blended[own].x, blended[own].y).distance(Vec2::new(4.0, 7.0)) < 1e-4);
}
