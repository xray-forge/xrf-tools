/// How a static geometry's vertices are laid out in their arena, as words in this order: position (three plain floats,
/// first, so an acceleration structure reads the arena in place), binormal, normal, tangent, base coordinate, and a word
/// the layout gives the meaning of. `shaders/static/vertex.wgsl` reads them by the same offsets.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StaticLayout {
  /// A sector's baked geometry: the base coordinate as two shorts, and a lightmap coordinate.
  Baked,
  /// A tree's: the base coordinate as four shorts (the last two its sway), and no lightmap.
  Tree,
  /// A spawned model's: the base coordinate as two floats, and no lightmap, its hemisphere from its place.
  Model,
}

impl StaticLayout {
  pub const ALL: [StaticLayout; 3] = [StaticLayout::Baked, StaticLayout::Tree, StaticLayout::Model];

  /// Layouts there are, which every per-layout array is sized by.
  pub const COUNT: usize = Self::ALL.len();

  /// Words a vertex takes: every layout comes to eight.
  pub const STRIDE: u32 = 8;

  /// Where each of a vertex's words stands: its position's three, then its binormal, normal and tangent, its base
  /// coordinate, and its layout's own word.
  pub const POSITION: usize = 0;
  pub const BINORMAL: usize = 3;
  pub const NORMAL: usize = 4;
  pub const TANGENT: usize = 5;
  pub const UV: usize = 6;
  pub const EXTRA: usize = 7;

  pub const fn get_index(self) -> usize {
    match self {
      StaticLayout::Baked => 0,
      StaticLayout::Tree => 1,
      StaticLayout::Model => 2,
    }
  }

  /// The static draws' vertex entry point that reads it.
  pub const fn get_vertex_entry(self) -> &'static str {
    match self {
      StaticLayout::Baked => "vs_baked",
      StaticLayout::Tree => "vs_tree",
      StaticLayout::Model => "vs_model",
    }
  }
}
