//! What `spawn build` refuses before it builds anything: a gamedata root without its level list, a level the list does
//! not name, a level directory missing its files, a destination that is no `.spawn` file, and a spawn already there
//! without `--force`; and that `--levels-root` and `--dest` move where the levels are read and the spawn written.

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
    String::from("--gamedata"),
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
  let error: String = get_error(run(&root, &[]));

  assert!(error.contains("game_levels.ltx"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn refuses_levels_it_cannot_build() -> XrfResult {
  let root: PathBuf = gamedata("levels")?;

  let unlisted: String = get_error(run(&root, &["--levels", "jupiter"]));

  assert!(
    unlisted.contains("Level jupiter is not one configs/game_levels.ltx lists"),
    "Unexpected error: {unlisted}"
  );

  let twice: String = get_error(run(&root, &["--levels", "zaton,zaton"]));

  assert!(
    twice.contains("Level zaton is asked for twice"),
    "Unexpected error: {twice}"
  );

  let empty: String = get_error(run(&root, &["--levels", "zaton"]));

  assert!(empty.contains("Level zaton: Not found"), "Unexpected error: {empty}");

  let none: String = get_error(run(&root, &[]));

  assert!(
    none.contains("No level the game lists holds"),
    "Unexpected error: {none}"
  );

  Ok(())
}

/// `-d` names the file written, as it does on every other command, so a folder or another extension is refused.
#[test]
fn refuses_a_destination_that_is_no_spawn_file() -> XrfResult {
  let root: PathBuf = gamedata("not-a-spawn")?;

  for dest in ["spawns", "spawns/all.ltx", "spawns/.spawn"] {
    let error: String = get_error(run(&root, &["--dest", dest]));

    assert!(error.contains("names no .spawn file"), "{dest}: {error}");
  }

  Ok(())
}

#[test]
fn reads_the_levels_and_writes_the_spawn_where_named() -> XrfResult {
  let root: PathBuf = gamedata("elsewhere")?;
  let built: PathBuf = root.join("built");
  let spawns: PathBuf = root.join("spawn");

  fs::create_dir_all(built.join("zaton"))?;

  for file in ["level.ai", "level.spawn", "level.game"] {
    fs::write(built.join("zaton").join(file), b"")?;
  }

  fs::create_dir_all(&spawns)?;
  fs::write(spawns.join("all.spawn"), [0u8; 4])?;

  let levels_root: String = built.display().to_string();

  // The spawn already there is looked for where the destination names it, not under spawns/.
  let error: String = get_error(run(&root, &["--dest", &spawns.join("all.spawn").display().to_string()]));

  assert!(error.contains("all.spawn"), "Unexpected error: {error}");

  // The levels are read from the directory named, where zaton is complete, so its empty files are what refuse.
  let error: String = get_error(run(
    &root,
    &[
      "--levels-root",
      &levels_root,
      "--dest",
      &spawns.join("test.spawn").display().to_string(),
    ],
  ));

  assert!(error.contains("Level zaton:"), "Unexpected error: {error}");
  assert!(!spawns.join("test.spawn").exists());

  Ok(())
}

#[test]
fn refuses_to_write_over_a_spawn_unless_forced() -> XrfResult {
  let root: PathBuf = gamedata("game-spawn")?;

  fs::create_dir_all(root.join("spawns"))?;
  fs::write(root.join("spawns/test.spawn"), [0u8; 4])?;

  let named: String = root.join("spawns/TEST.spawn").display().to_string();
  let error: String = get_error(run(&root, &["--dest", &named]));

  assert!(error.contains("test.spawn"), "Unexpected error: {error}");
  assert_eq!(fs::read(root.join("spawns/test.spawn"))?, [0u8; 4]);

  // Forced, it goes on to the levels.
  let forced: String = get_error(run(&root, &["--dest", &named, "--force"]));

  assert!(
    forced.contains("No level the game lists holds"),
    "Unexpected error: {forced}"
  );

  Ok(())
}
