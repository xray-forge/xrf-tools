/// A texture lit implicitly ([`crate::ImplicitLighter`]): its name as the compilers find it, its size, its texels with
/// each one's hemisphere in alpha, and its map of static light and sun, both row by row, which the level keeps as
/// `<name>.dds` and `<name>_lm.dds`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImplicitMap {
  pub name: String,
  pub width: u32,
  pub height: u32,
  pub base: Vec<[u8; 4]>,
  pub lightmap: Vec<[u8; 4]>,
}
