use xrf_math::EPS;

/// Whether a straight line across the ground crosses a box on it, as xrAI tests each AI map cell a straight walk
/// steps into, in both the cover pass and the root spawn build.
pub struct Box2Pick;

impl Box2Pick {
  /// `Fbox2::pick_exact` (`xrCore/_fbox2.h`): whether the line through `start` along `direction`, both ways, meets
  /// the box from `min` to `max` on one of its four sides, within `EPS`, in single precision as the engine computes it.
  pub fn is_crossing(min: (f32, f32), max: (f32, f32), start: (f32, f32), direction: (f32, f32)) -> bool {
    let low: (f32, f32) = (min.0 - start.0, min.1 - start.1);
    let high: (f32, f32) = (max.0 - start.0, max.1 - start.1);

    if direction.0 != 0.0 {
      for side in [low.0, high.0] {
        let y: f32 = side / direction.0 * direction.1;

        if y >= low.1 - EPS && y <= high.1 + EPS {
          return true;
        }
      }
    }

    if direction.1 != 0.0 {
      for side in [low.1, high.1] {
        let x: f32 = side / direction.1 * direction.0;

        if x >= low.0 - EPS && x <= high.0 + EPS {
          return true;
        }
      }
    }

    false
  }
}
