/// A window's width and height; each use says in which units.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct PlatformSize {
  pub width: u32,
  pub height: u32,
}
