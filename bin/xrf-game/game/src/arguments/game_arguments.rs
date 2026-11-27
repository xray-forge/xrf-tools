use clap::Parser;
use xrf_game_settings::GamePreset;

/// What the game is started with: the settings for this run over its settings file.
#[derive(Clone, Debug, Default, Parser)]
#[command(name = "xrf-game", version, about = "The XRF game")]
pub struct GameArguments {
  /// The preset to start from in place of the settings file's.
  #[arg(long)]
  pub preset: Option<GamePreset>,
  /// One setting for this run, its dotted path and value, as `view.shadows.resolution=4096`; repeated for several,
  /// applied over the settings file in order and never written back.
  #[arg(long = "set", value_name = "PATH=VALUE")]
  pub sets: Vec<String>,
}
