use std::time::Duration;

use crate::light::light_bake_census::LightBakeCensus;

/// How a level's light was baked, for the light step's report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightBake {
  pub census: LightBakeCensus,
  /// The GPU that traced the lightmaps' and implicit maps' shadow rays, none where the CPU did.
  pub gpu: Option<String>,
  /// How long the bake took, within its geometry compile.
  pub duration: Duration,
}
