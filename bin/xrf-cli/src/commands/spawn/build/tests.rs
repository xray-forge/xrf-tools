//! What `spawn build` refuses before it builds anything: a gamedata root without its level list, a level the list does
//! not name, a level directory missing its files, and the game's own spawn without `--force`.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::commands::spawn::build::command::BuildCommand;
use crate::core::command::CommandResult;
use crate::core::command::command_testing::run_command;

fn run(root: &Path, extra: &[&str]) -> CommandResult {
  let mut arguments: Vec<String> = vec![
    String::from("build"),
    String::from("--path"),
    root.display().to_string(),
    String::from("--silent"),
  ];

  arguments.extend(extra.iter().map(|it| String::from(*it)));

  run_command(&BuildCommand, &arguments)
}

fn get_error(verdict: CommandResult) -> String {
  match verdict {
    Ok(()) => panic!("Expect the build to be refused"),
    Err(error) => error.to_string(),
  }
}

/// A gamedata root listing one level, `zaton`, whose directory is empty.
fn gamedata(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&format!("spawn-build/{name}"))?;

  fs::create_dir_all(root.join("configs"))?;
  fs::create_dir_all(root.join("levels/zaton"))?;
  fs::write(
    root.join("configs/game_levels.ltx"),
    "[levels]\nlevel108\n\n[level108]\nname = zaton\ncaption = \"zaton\"\noffset = 12000.0, 1000.0, 10000.0\nid = 108\n",
  )?;

  Ok(root)
}

#[test]
fn refuses_a_root_without_its_level_list() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("spawn-build/no-list")?;
  let error: String = get_error(run(&root, &["--out", "test"]));

  assert!(error.contains("game_levels.ltx"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn refuses_levels_it_cannot_build() -> XrfResult {
  let root: PathBuf = gamedata("levels")?;

  let unlisted: String = get_error(run(&root, &["--out", "test", "--levels", "jupiter"]));

  assert!(
    unlisted.contains("Level jupiter is not one configs/game_levels.ltx lists"),
    "Unexpected error: {unlisted}"
  );

  let twice: String = get_error(run(&root, &["--out", "test", "--levels", "zaton,zaton"]));

  assert!(
    twice.contains("Level zaton is asked for twice"),
    "Unexpected error: {twice}"
  );

  let empty: String = get_error(run(&root, &["--out", "test", "--levels", "zaton"]));

  assert!(empty.contains("Level zaton: Not found"), "Unexpected error: {empty}");

  let none: String = get_error(run(&root, &["--out", "test"]));

  assert!(
    none.contains("No level the game lists holds"),
    "Unexpected error: {none}"
  );

  Ok(())
}

#[test]
fn refuses_to_write_over_the_games_spawn_unless_forced() -> XrfResult {
  let root: PathBuf = gamedata("game-spawn")?;

  fs::create_dir_all(root.join("spawns"))?;
  fs::write(root.join("spawns/all.spawn"), [0u8; 4])?;

  let error: String = get_error(run(&root, &["--out", "all"]));

  assert!(error.contains("is the game's own spawn"), "Unexpected error: {error}");
  assert_eq!(fs::read(root.join("spawns/all.spawn"))?, [0u8; 4]);

  // Forced, it goes on to the levels.
  let forced: String = get_error(run(&root, &["--out", "all", "--force"]));

  assert!(
    forced.contains("No level the game lists holds"),
    "Unexpected error: {forced}"
  );

  Ok(())
}
