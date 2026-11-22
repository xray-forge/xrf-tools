use xrf_math::glam::Vec3;

/// A path link as helpers draw it: a line with an arrowhead at its target.
pub(crate) struct HelperLinkLines;

impl HelperLinkLines {
  /// Arrowhead barbs: the longest in metres, the longest as a share of the link, and the angle off the link.
  const ARROW_LENGTH: f32 = 0.6;
  const ARROW_SHARE: f32 = 0.25;
  const ARROW_ANGLE: f32 = 0.45;

  /// The line from `from` to `to` and two arrowhead barbs at `to`, turned about the vertical; none for a zero-length
  /// link.
  pub(crate) fn list(from: Vec3, to: Vec3) -> Vec<[[f32; 3]; 2]> {
    let span: Vec3 = to - from;
    let length: f32 = span.length();

    if length <= f32::EPSILON {
      return Vec::new();
    }

    let back: Vec3 = -span / length * (length * Self::ARROW_SHARE).min(Self::ARROW_LENGTH);
    let barb = |angle: f32| -> [[f32; 3]; 2] {
      let (sin, cos): (f32, f32) = angle.sin_cos();

      [
        to.to_array(),
        (to + Vec3::new(back.x * cos - back.z * sin, back.y, back.x * sin + back.z * cos)).to_array(),
      ]
    };

    vec![
      [from.to_array(), to.to_array()],
      barb(Self::ARROW_ANGLE),
      barb(-Self::ARROW_ANGLE),
    ]
  }
}
