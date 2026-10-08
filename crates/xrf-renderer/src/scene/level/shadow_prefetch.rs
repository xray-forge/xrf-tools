use crate::lighting::sun_cascade::SunCascade;

/// A cascade's next static layer, drawn band by band into its hidden layer ahead of the turn that would stale the one
/// shown: the fit it is drawn at, what the scene held when its first band was drawn, the bands drawn, and the frames a
/// band has waited.
#[derive(Clone, Copy, Debug)]
pub struct ShadowPrefetch {
  pub fit: SunCascade,
  pub contents: usize,
  pub bands: u32,
  pub waited: u32,
}
