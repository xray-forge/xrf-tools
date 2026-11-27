/// A loading view the developer layer draws over the whole window while the game's level streams in, so the level is
/// not seen being built around the camera: what loads, how far along, from nothing to whole, and a line of detail. The
/// developer layer's stand-in until the game has its own interface.
#[derive(Clone, Debug, PartialEq)]
pub struct DeveloperLoading {
  pub title: String,
  /// From 0 to 1.
  pub progress: f32,
  pub detail: String,
}
