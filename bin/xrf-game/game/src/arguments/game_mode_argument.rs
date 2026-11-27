use clap::ValueEnum;

/// How the player moves, as `--mode` names it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum GameModeArgument {
  /// A free camera through everything.
  #[default]
  Fly,
  /// On foot, on a physics engine: not built yet.
  Walk,
}
