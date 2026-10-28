/// Which of a view's per-frame targets its frame writes, each made in the frame's graph only where it does: the
/// ambient occlusion pair while an occlusion is drawn, the haze map while the sky is hazed, and the engine bloom's pair
/// while the engine bloom is built, shown or only blurred towards by the distortion.
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewTargetUses {
  pub is_occluded: bool,
  pub is_hazed: bool,
  pub is_engine_bloomed: bool,
}
