use std::path::PathBuf;

use clap::Parser;
use xrf_game_settings::GamePreset;

use crate::arguments::game_mode_argument::GameModeArgument;

/// What the game is started with: the game tree and level to play, how the player moves, and the settings for this run
/// over its settings file.
#[derive(Clone, Debug, Default, Parser)]
#[command(name = "xrf-game", version, about = "The XRF game")]
pub struct GameArguments {
  /// The X-Ray game tree to play: its root folder, unpacked `gamedata` or packed archives, as the tools mount it.
  #[arg(long)]
  pub root: Option<PathBuf>,
  /// The level to open, by its name in the tree (`zaton`); without one the game lists the levels it found and exits.
  #[arg(long)]
  pub level: Option<String>,
  /// Takes the control endpoint's requests on standard input and answers on standard output, as the MCP bridge
  /// (`xrf-game-mcp`) starts the game; its input ending quits the game.
  #[arg(long)]
  pub mcp: bool,
  /// Listens on a loopback port for the MCP bridge to attach to this game, started by hand, its `game_attach` tool
  /// reading how from `attach.json` in the game's own folder; a bridge leaving quits nothing. Ignored with `--mcp`.
  #[arg(long)]
  pub listen: bool,
  /// How the player moves.
  #[arg(long, value_enum, default_value_t)]
  pub mode: GameModeArgument,
  /// The preset to start from in place of the settings file's.
  #[arg(long)]
  pub preset: Option<GamePreset>,
  /// The new game's seed, every random draw of the simulation keyed by it, so the same seed starts the same game; a
  /// new one each run without it, logged so a run can be repeated.
  #[arg(long)]
  pub seed: Option<u64>,
  /// One setting for this run, its dotted path and value, as `view.shadows.resolution=4096`; repeated for several,
  /// applied over the settings file in order and never written back.
  #[arg(long = "set", value_name = "PATH=VALUE")]
  pub sets: Vec<String>,
}
