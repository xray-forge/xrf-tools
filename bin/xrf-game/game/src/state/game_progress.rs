use xrf_game_alife::Alife;
use xrf_game_objects::{ObjectId, ObjectRegistry};

use crate::time::game_time::GameTime;

/// The game as a save holds it, kept from level to level: the seed every random draw is keyed by, the game's time,
/// every object, A-Life, and which object is the player.
#[derive(Clone, Debug, PartialEq)]
pub struct GameProgress {
  pub seed: u64,
  pub time: GameTime,
  pub objects: ObjectRegistry,
  pub alife: Alife,
  /// The player's object: the registry holds its record for as long as the game runs.
  pub player: ObjectId,
}
