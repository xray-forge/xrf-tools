/// One glow a lens flare draws over the frame, `CLensFlareDescriptor::SFlare`: a flare along the line from the sun
/// through the screen's centre, or the gradient about the sun.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderFlare {
  /// By reference, as a keyframe's textures are.
  pub texture: String,
  /// Half its side, as a share of the distance it is drawn at.
  pub radius: f32,
  pub opacity: f32,
  /// Where along the line it stands: one at the sun, zero at the screen's centre; the gradient stands at the sun.
  pub position: f32,
}
