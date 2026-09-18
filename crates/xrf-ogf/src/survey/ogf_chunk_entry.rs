/// One chunk found while walking an ogf file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfChunkEntry {
  pub id: u32,
  /// 0 for a chunk of the root object, 1 for a chunk of a direct child, and so on.
  pub depth: usize,
  pub size: u64,
}
