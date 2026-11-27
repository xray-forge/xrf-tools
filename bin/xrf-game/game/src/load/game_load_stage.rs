/// A step of loading a level, as the loading view names it and the log times it. A load declares the steps it takes:
/// the game's first launch starts the renderer, a later load of another level does not.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum GameLoadStage {
  /// The game tree mounted and its engine found.
  Tree,
  /// What the game starts with: the spawn's objects, the simulation and the player's configs.
  Game,
  /// The level read as the world shows it.
  Level,
  /// The level's collision built for the physics world.
  Physics,
  /// The renderer started: its device, its passes and DLSS, until it draws.
  Renderer,
  /// The level's sectors and textures put on the GPU, until every one is settled.
  Streaming,
}

impl GameLoadStage {
  /// Every stage of the game's first launch, in the order they begin.
  pub(crate) const LAUNCH: [Self; 6] = [
    Self::Tree,
    Self::Game,
    Self::Level,
    Self::Physics,
    Self::Renderer,
    Self::Streaming,
  ];

  /// What the loading view says while it runs.
  pub(crate) fn get_title(self) -> &'static str {
    match self {
      Self::Tree => "Mounting the game",
      Self::Game => "Starting a new game",
      Self::Level => "Reading the level",
      Self::Physics => "Building the level's collision",
      Self::Renderer => "Starting the renderer",
      Self::Streaming => "Streaming the level",
    }
  }
}
