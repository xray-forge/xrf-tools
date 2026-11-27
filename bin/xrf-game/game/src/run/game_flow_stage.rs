/// Where the main thread's flow stands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameFlowStage {
  /// No game runs and none loads.
  Menu,
  /// A load runs: its view covers the window and the player is held, the game opening or opened under it.
  Loading,
  /// A game is played.
  Playing,
}
