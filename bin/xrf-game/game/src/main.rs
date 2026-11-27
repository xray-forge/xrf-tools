#![doc = include_str!("../README.md")]

use std::process::ExitCode;

use clap::Parser;
use xrf_game::{GameArguments, GameDirectories, GameLogSetup};
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
  log::info!(
    "Exiting after {:.1?}: the window comes with the first movement mode",
    ProcessStart::get_uptime()
  );

  ExitCode::SUCCESS
}
