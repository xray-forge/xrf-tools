//! Exercises the `ltx format` command end to end.

use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;

use crate::commands::ltx::format::command::FormatCommand;
use crate::core::generic_command::{CommandResult, GenericCommand};

fn create_root(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = std::env::temp_dir().join(format!("xrf-cli-format-ltx-{name}-{}", std::process::id()));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&root)?;

  Ok(root)
}

#[test]
fn preserves_standalone_semicolon_comments() -> CommandResult {
  let root: PathBuf = create_root("standalone-comment")?;
  let file: PathBuf = root.join("comment.ltx");

  fs::write(&file, ";\n")?;

  let command: FormatCommand = FormatCommand;
  let matches = command
    .init()
    .try_get_matches_from(["format", "--path", &file.display().to_string(), "--silent"])?;

  command.execute(&matches)?;

  assert_eq!(fs::read_to_string(&file)?, ";\r\n");

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn formatting_standalone_comment_is_idempotent() -> CommandResult {
  let root: PathBuf = create_root("idempotent-comment")?;
  let file: PathBuf = root.join("comment.ltx");
  let expected: Vec<u8> = b";\r\n".to_vec();

  fs::write(&file, &expected)?;

  let command: FormatCommand = FormatCommand;
  let matches = command
    .init()
    .try_get_matches_from(["format", "--path", &file.display().to_string(), "--silent"])?;

  command.execute(&matches)?;
  let formatted_once: Vec<u8> = fs::read(&file)?;

  command.execute(&matches)?;
  let formatted_twice: Vec<u8> = fs::read(&file)?;

  assert_eq!(formatted_once, expected);
  assert_eq!(formatted_twice, formatted_once);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn formats_the_loose_configs_of_an_installation() -> CommandResult {
  // The user-facing shape of the write rule: point the command at an installation and only its loose files change.
  let root: PathBuf = create_root("installation-format")?;
  let config: PathBuf = root.join("gamedata/configs/system.ltx");

  fs::write(
    root.join("fsgame.ltx"),
    "$arch_dir$ = false | false | $fs_root$ | db\\\n$game_data$ = true | true | $fs_root$ | gamedata\\\n",
  )?;
  fs::create_dir_all(config.parent().expect("config parent"))?;
  fs::write(&config, ";\n")?;

  let command: FormatCommand = FormatCommand;
  let matches = command
    .init()
    .try_get_matches_from(["format", "--path", &root.display().to_string(), "--silent"])?;

  command.execute(&matches)?;

  assert_eq!(fs::read_to_string(&config)?, ";\r\n");

  fs::remove_dir_all(root)?;

  Ok(())
}
