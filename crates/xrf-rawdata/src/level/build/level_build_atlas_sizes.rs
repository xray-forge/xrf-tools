/// The sizes an atlas may take, width and height: it starts at the smallest and doubles its narrower side up to the
/// largest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LevelBuildAtlasSizes {
  pub(crate) smallest: (u32, u32),
  pub(crate) largest: (u32, u32),
}
