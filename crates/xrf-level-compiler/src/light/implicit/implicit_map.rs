/// A texture lit implicitly ([`crate::light::implicit::implicit_lighter::ImplicitLighter`]): its name as the compilers
/// find it, its size, its texels with each one's hemisphere in alpha, and its map of static light and sun, both row by
/// row, which the level keeps as `<name>.dds` and `<name>_lm.dds`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ImplicitMap {
  pub(crate) name: String,
  pub(crate) width: u32,
  pub(crate) height: u32,
  pub(crate) base: Vec<[u8; 4]>,
  pub(crate) lightmap: Vec<[u8; 4]>,
}
