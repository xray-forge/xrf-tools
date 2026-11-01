/// The indirect light the occlusion's search gathers beside it; the default gathers none.
#[derive(Clone, Copy, Debug, Default)]
pub struct VbaoLight {
  /// How strong it is; none gathers no light.
  pub intensity: f32,
  /// The metres around it is gathered from, at least the occlusion's radius.
  pub radius: f32,
  /// The steps it takes past the occlusion's.
  pub steps: u32,
}
