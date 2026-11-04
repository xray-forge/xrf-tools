/// How loud a placed sound is heard at a distance, from its minimum and maximum distance: full up to the minimum, then
/// falling as one over the distance does, eased to silence over the last fifth of its range so nothing is cut off
/// where a sound leaves it. One curve, whether the acoustics run or sound plays dry.
pub(crate) struct DistanceCurve;

impl DistanceCurve {
  /// The share of a sound's range, from its maximum distance in, over which it eases to silence.
  const TAPER: f32 = 0.2;

  /// The gain at `distance` metres of a sound heard fully up to `min_distance` and not at all past `max_distance`.
  pub(crate) fn get_attenuation(min_distance: f32, max_distance: f32, distance: f32) -> f32 {
    let min_distance: f32 = min_distance.max(0.01);
    let max_distance: f32 = max_distance.max(min_distance);

    if !distance.is_finite() || distance >= max_distance {
      return 0.0;
    }

    let inverse: f32 = (min_distance / distance.max(min_distance)).min(1.0);
    let taper: f32 = (max_distance - min_distance) * Self::TAPER;
    let left: f32 = if taper > 0.0 {
      ((max_distance - distance) / taper).clamp(0.0, 1.0)
    } else {
      1.0
    };

    inverse * left * left * (3.0 - 2.0 * left)
  }
}
