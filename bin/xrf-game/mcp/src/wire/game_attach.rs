use serde::{Deserialize, Serialize};

/// How a bridge attaches to a game started by hand with `--listen`, as the game writes it to `attach.json` in its own
/// folder while it runs: the loopback port it listens on, the token a connection's first line must be, and which game
/// it is.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GameAttach {
  pub port: u16,
  /// This run's token, new each run.
  pub token: String,
  /// The game's process id.
  pub process: u32,
  /// The level it shows.
  pub level: String,
}

impl GameAttach {
  /// The file's name in the game's own folder.
  pub const FILE_NAME: &'static str = "attach.json";
}
