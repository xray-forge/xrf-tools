use clap::ValueEnum;

/// How the player moves, as `--mode` names it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum GameModeArgument {
  /// A free camera through everything.
  #[default]
  Fly,
  /// On foot, the actor walking the level's collision on the physics engine.
  Walk,
}
