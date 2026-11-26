#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum PlatformWindowMode {
  /// A framed window the user can move and resize.
  #[default]
  Windowed,
  /// Borderless over the whole display it is on, at the display's own mode.
  Fullscreen,
}
