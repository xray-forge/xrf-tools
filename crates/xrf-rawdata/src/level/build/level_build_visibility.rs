/// How a sphere lies against a sector's volume, `EVisible` as `CSector::Intersect` gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LevelBuildVisibility {
  None,
  PartialOutside,
  PartialInside,
  Fully,
}
