#![doc = include_str!("../README.md")]

use std::process::ExitCode;

use clap::Parser;
use xrf_game::{GameArguments, GameDirectories, GameLogSetup, GameRun};
use xrf_game_settings::GameSettings;
use xrf_logging::ProcessStart;

fn main() -> ExitCode {
  ProcessStart::stamp();

  let arguments: GameArguments = GameArguments::parse();
  let directories: GameDirectories = GameDirectories::for_game();

  GameLogSetup::install(&directories, env!("CARGO_PKG_VERSION"));

  let settings: GameSettings = GameSettings::load(&directories.get_settings_files(), arguments.preset, &arguments.sets);

  log::info!(
    "Settings resolved: window {:?} {}x{}, antialiasing {:?}, shadows {} at {}",
    settings.window.mode,
    settings.window.width,
    settings.window.height,
    settings.view.antialiasing,
    if settings.view.shadows.is_enabled { "on" } else { "off" },
    settings.view.shadows.resolution
  );

  match GameRun::run(&arguments, &settings, &directories) {
    Ok(()) => {
      log::info!("Exiting after {:.1?}", ProcessStart::get_uptime());

      ExitCode::SUCCESS
    }
    #[expect(
      clippy::print_stderr,
      reason = "a run started from a terminal says why it cannot start where it was started, not only in its log"
    )]
    Err(error) => {
      log::error!("The game cannot run: {error}");
      eprintln!("xrf-game: {error}");

      ExitCode::FAILURE
    }
  }
}
