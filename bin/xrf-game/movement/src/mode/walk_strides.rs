use xrf_math::glam::Vec3;

use crate::event::movement_step::MovementStep;
use crate::mode::movement_footing::MovementFooting;
use crate::mode::walk_pace::WalkPace;

/// How far the walking feet went along the ground since their last step, a step told each stride of the pace walked.
/// Counted every tick, so no step is lost or doubled however rarely presentation looks.
///
/// X-Ray times a step by the leg animation's cycle (`CStepManager`, `step_manager.cpp`); with no actor animations yet,
/// the stride of each pace stands in.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WalkStrides {
  /// Where the feet stood on the ground at the last tick; none in the air or flying.
  feet: Option<Vec3>,
  /// How far they went along the ground since the last step.
  walked: f32,
}

impl WalkStrides {
  /// The speed below which the feet count as standing, so the physics' settling takes no step.
  const STANDING_SPEED: f32 = 0.2;

  /// The step `footing` finishes, if it finishes one: none while flying, in the air or standing.
  pub fn take(&mut self, footing: Option<&MovementFooting>) -> Option<MovementStep> {
    let Some(footing) = footing else {
      *self = Self::default();

      return None;
    };
    let Some(ground) = footing.ground_material else {
      self.feet = None;

      return None;
    };
    let previous: Option<Vec3> = self.feet.replace(footing.feet);

    if footing.speed < Self::STANDING_SPEED {
      return None;
    }

    if let Some(previous) = previous {
      self.walked += (footing.feet - previous).with_y(0.0).length();
    }

    let stride: f32 = Self::get_stride(footing.pace);

    if self.walked < stride {
      return None;
    }

    self.walked -= stride;

    Some(MovementStep {
      feet: footing.feet,
      ground_material: ground,
      pace: footing.pace,
    })
  }

  /// How far the feet go between two steps at `pace`.
  pub const fn get_stride(pace: WalkPace) -> f32 {
    match pace {
      WalkPace::Walk => 0.65,
      WalkPace::Run => 1.0,
      WalkPace::Sprint => 1.6,
      WalkPace::Crouch => 0.5,
    }
  }
}
