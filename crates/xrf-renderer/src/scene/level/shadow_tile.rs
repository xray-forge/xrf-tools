/// A square of the light shadow atlas, in texels: its corner and side.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ShadowTile {
  pub x: u32,
  pub y: u32,
  pub size: u32,
}
