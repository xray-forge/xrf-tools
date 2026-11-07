//! That a key's rotation comes back from the angles the decompile keys it by, at a straight up or down pitch too.

use xrf_math::{Affine3d, Quaternion};

use crate::object::decompile::skeleton::motion_converter::MotionConverter;

/// One quantisation step of a 16-bit key's component.
const STEP: f32 = 1.0 / 32767.0;

/// How far `rotation` comes back from its angles, through the compile's conversion, in quantisation steps; `q` and
/// `-q` being one rotation.
fn steps_back(rotation: Quaternion) -> f32 {
  let length: f32 =
    (rotation.x * rotation.x + rotation.y * rotation.y + rotation.z * rotation.z + rotation.w * rotation.w).sqrt();
  let unit = [rotation.x, rotation.y, rotation.z, rotation.w].map(|it| it / length);
  let back: Quaternion = Affine3d::from_xyz_inverse(&MotionConverter::get_xyz_inverse(&rotation)).get_rotation();
  let back = [back.x, back.y, back.z, back.w];
  let gap = |sign: f32| {
    unit
      .iter()
      .zip(back)
      .map(|(a, b)| (a - sign * b).abs())
      .fold(0.0, f32::max)
  };

  gap(1.0).min(gap(-1.0)) / STEP
}

#[test]
fn keys_a_rotation_by_angles_that_give_it_back_within_a_step() {
  // A vanilla key of `wpn_artefact_hud`'s idle, its pitch 0.25 degrees short of straight down, as dequantised; worked
  // in single precision its angles gave it back 73 steps off.
  let near_the_pole = Quaternion {
    x: -0.043_610_95,
    y: -0.705_740_5,
    z: 0.705_740_5,
    w: 0.043_641_47,
  };
  let plain = Quaternion {
    x: 0.1,
    y: 0.2,
    z: 0.3,
    w: 0.927_361_8,
  };

  for rotation in [near_the_pole, plain] {
    let steps: f32 = steps_back(rotation);

    assert!(steps <= 1.0, "{rotation:?} came back {steps} steps off");
  }
}
