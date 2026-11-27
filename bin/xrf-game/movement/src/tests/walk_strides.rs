use xrf_math::glam::Vec3;

use crate::event::movement_step::MovementStep;
use crate::mode::movement_footing::MovementFooting;
use crate::mode::walk_pace::WalkPace;
use crate::mode::walk_strides::WalkStrides;

const DIRT: u16 = 2;

fn footing(x: f32, ground: Option<u16>, pace: WalkPace) -> MovementFooting {
  MovementFooting {
    feet: Vec3::new(x, 0.0, 0.0),
    ground_material: ground,
    pace,
    speed: 2.77,
  }
}

/// Walks the feet from 0 to `distance` metres in steps of a tenth, gathering the steps they take.
fn walk(strides: &mut WalkStrides, distance: f32, ground: Option<u16>, pace: WalkPace) -> Vec<MovementStep> {
  (0..=(distance * 10.0).round() as u32)
    .filter_map(|tenth| strides.take(Some(&footing(tenth as f32 / 10.0, ground, pace))))
    .collect()
}

#[test]
fn takes_a_step_each_stride_of_its_pace() {
  let ran: Vec<MovementStep> = walk(&mut WalkStrides::default(), 4.5, Some(DIRT), WalkPace::Run);

  assert_eq!(ran.len(), 4, "a running stride is a metre");
  assert!(
    ran
      .iter()
      .all(|step| step.ground_material == DIRT && step.pace == WalkPace::Run)
  );
  assert_eq!(
    walk(&mut WalkStrides::default(), 6.8, Some(DIRT), WalkPace::Walk).len(),
    10
  );
  assert_eq!(
    walk(&mut WalkStrides::default(), 6.8, Some(DIRT), WalkPace::Sprint).len(),
    4
  );
}

#[test]
fn takes_none_in_the_air_flying_or_standing() {
  let mut strides: WalkStrides = WalkStrides::default();

  assert!(walk(&mut strides, 4.0, None, WalkPace::Run).is_empty());
  assert!(strides.take(None).is_none());

  let standing: MovementFooting = MovementFooting {
    speed: 0.0,
    ..footing(0.0, Some(DIRT), WalkPace::Run)
  };

  assert!((0..100).all(|_| strides.take(Some(&standing)).is_none()));
}
