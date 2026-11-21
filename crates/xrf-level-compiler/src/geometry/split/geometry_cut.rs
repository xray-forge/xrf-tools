use crate::geometry::split::geometry_splits::GeometrySplits;
use crate::light::lightmap::chart::light_chart::LightChart;

/// A level's splits before they are joined, and the lightmap charts they were cut by ([`GeometrySplits::cut`]).
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryCut {
  pub(crate) splits: GeometrySplits,
  /// The charts in xrLC's order of its deflectors; none where no charts are cut.
  pub(crate) charts: Vec<LightChart>,
}
