/// How a static geometry's vertices are laid out in their arena, as words in the order the renderer's TypeScript
/// predecessor sorted them: binormal, normal, tangent, base coordinate, lightmap coordinate, position.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StaticLayout {
  /// A sector's baked geometry: the base coordinate as two shorts, and a lightmap coordinate.
  Baked,
  /// A tree's: the base coordinate as four shorts (the last two its sway), and no lightmap.
  Tree,
}

impl StaticLayout {
  pub const ALL: [StaticLayout; 2] = [StaticLayout::Baked, StaticLayout::Tree];

  /// Words a vertex takes: both layouts come to eight.
  pub const STRIDE: u32 = 8;

  pub const fn get_index(self) -> usize {
    match self {
      StaticLayout::Baked => 0,
      StaticLayout::Tree => 1,
    }
  }
}
