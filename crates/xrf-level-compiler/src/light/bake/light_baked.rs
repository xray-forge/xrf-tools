use crate::geometry::split::geometry_splits::GeometrySplits;
use crate::light::bake::light_bake_result::LightBakeResult;

/// What baking a level's light leaves ([`crate::light::bake::light_baker::LightBaker::bake`]): its splits, each chart
/// cut into a split of its own, and the light.
pub(crate) struct LightBaked {
  pub(crate) splits: GeometrySplits,
  pub(crate) light: LightBakeResult,
}
