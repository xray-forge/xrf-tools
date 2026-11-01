/// A square of the light shadow atlas, in texels: its corner and side.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ShadowTile {
  pub x: u32,
  pub y: u32,
  pub size: u32,
}

impl ShadowTile {
  /// Its region of the atlas: texels across, down, wide and high.
  pub fn get_region(self) -> [u32; 4] {
    [self.x, self.y, self.size, self.size]
  }
}
