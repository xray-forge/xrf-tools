/// Which game a load opens on its level.
// todo: a saved game, once saves are written and read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameLoadSource {
  /// A new game, every random draw keyed by `seed`, so the same seed starts the same game.
  New { seed: u64 },
}
