use glam::Vec3;
use xrf_visual::LightAnimatorDescription;

/// `CLAItem::CalculateRGB`: the colour an animation holds at a time, each channel in `[0, 255]`, looping over its frames.
pub fn to_animated_color(animator: &LightAnimatorDescription, seconds: f32) -> Vec3 {
  let length: f32 = animator.frame_count as f32 / animator.fps;
  let frame: f32 = if length > 0.0 && length.is_finite() {
    ((seconds % length) * animator.fps).floor()
  } else {
    0.0
  };

  to_interpolated_color(animator, frame)
}

/// `CLAItem::InterpolateRGB`: a key's own colour on its frame, the last key's past the last, and between two keys the
/// two blended by how far the frame stands between them.
pub fn to_interpolated_color(animator: &LightAnimatorDescription, frame: f32) -> Vec3 {
  let keys = &animator.keys;
  let next: Option<usize> = keys.iter().position(|key| key.frame as f32 > frame);

  match (next, keys.first(), keys.last()) {
    (_, None, _) | (_, _, None) => Vec3::ZERO,
    // Past the last key, or on the first one with nothing before it to blend from.
    (None, _, Some(last)) => Vec3::from(last.color),
    (Some(0), Some(first), _) => Vec3::from(first.color),
    (Some(next), _, _) => {
      let (from, to) = (&keys[next - 1], &keys[next]);
      let blend: f32 = (frame - from.frame as f32) / (to.frame as f32 - from.frame as f32);

      Vec3::from(from.color).lerp(Vec3::from(to.color), blend)
    }
  }
}
