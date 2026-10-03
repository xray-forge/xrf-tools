use crate::host::render_thunderbolt_gradient::RenderThunderboltGradient;

/// A bolt of a collection, as `SThunderboltDesc` loads it.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderThunderbolt {
  /// Its model, by index among the thunder's models; none for one that did not read, which draws nothing.
  pub model: Option<usize>,
  /// Its `color_anim`, by index among the thunder's animators; none for one the library lacks, which flashes black.
  pub color: Option<usize>,
  pub top: RenderThunderboltGradient,
  pub center: RenderThunderboltGradient,
}
