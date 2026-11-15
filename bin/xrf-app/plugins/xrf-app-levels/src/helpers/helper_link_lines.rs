use glam::Vec3;

/// A path's link as the helpers draw it: its line and an arrowhead at the point it leads to.
pub(crate) struct HelperLinkLines;

impl HelperLinkLines {
  /// An arrowhead's barbs: metres at most, a share of the link at most, and their angle off the link.
  const ARROW_LENGTH: f32 = 0.6;
  const ARROW_SHARE: f32 = 0.25;
  const ARROW_ANGLE: f32 = 0.45;

  /// A link from `from` to `to` as its line and an arrowhead's two barbs at `to`, turned about the vertical; none for
  /// a link of no length.
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
