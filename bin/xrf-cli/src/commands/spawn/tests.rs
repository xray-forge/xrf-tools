use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_spawn::AnySpawnFile;
use xrf_test_utils::GeneratedResources;

use crate::commands::packed_file::{RepackCommand, UnpackCommand, VerifyCommand};
use crate::commands::spawn::fixtures::{new_level_object, write_level_spawn_list};
use crate::core::command::command_testing::run_command;
use crate::core::command::{CommandError, CommandResult, GenericCommand};

fn run<T: GenericCommand>(command: &T, path: &Path, extra: &[&str]) -> CommandResult {
  let mut arguments: Vec<String> = vec![
    String::from(command.operation()),
    String::from("--path"),
    path.display().to_string(),
    String::from("--silent"),
  ];

  arguments.extend(extra.iter().map(|it| String::from(*it)));

  run_command(command, &arguments)
}

fn new_level_spawn_list(directory: &str) -> XrfResult<(PathBuf, Vec<u8>)> {
  let path: PathBuf = GeneratedResources::create_dir(directory)?.join("level.spawn");
  let bytes: Vec<u8> = write_level_spawn_list(
    &path,
    vec![
      new_level_object(0, "graph_point", "fake_graph_point"),
      new_level_object(1, "m_stalker", "esc_wolf"),
    ],
  )?;

  Ok((path, bytes))
}

#[test]
fn verifies_a_level_spawn_list_as_readable() -> XrfResult {
  let (path, _) = new_level_spawn_list("spawn/verify_level")?;

  run(&VerifyCommand::<AnySpawnFile>::default(), &path, &[]).expect("a level's spawn list reads");

  Ok(())
}

#[test]
fn judges_a_file_of_neither_kind() -> XrfResult {
  let path: PathBuf = GeneratedResources::create_dir("spawn/verify_neither")?.join("container.spawn");

  // One chunk holding four bytes: a container, but neither a root's header nor a spawn packet.
  fs::write(&path, [7, 0, 0, 0, 4, 0, 0, 0, 1, 2, 3, 4])?;

  let error: CommandError = run(&VerifyCommand::<AnySpawnFile>::default(), &path, &[]).expect_err("judged");

  assert_eq!(error.exit_code(), 3);

  Ok(())
}

#[test]
fn repacks_a_level_spawn_list_byte_for_byte() -> XrfResult {
  let (path, bytes) = new_level_spawn_list("spawn/repack_level")?;
  let dest: PathBuf = path.with_file_name("repacked.spawn");

  run(
    &RepackCommand::<AnySpawnFile>::default(),
    &path,
    &["--dest", &dest.display().to_string(), "--force"],
  )
  .expect("a level's spawn list repacks");

  assert_eq!(fs::read(&dest)?, bytes);

  Ok(())
}

/// A level's spawn list has no text form yet, which the refusal says rather than misreading it as a root spawn.
#[test]
fn refuses_to_unpack_a_level_spawn_list_saying_why() -> XrfResult {
  let (path, _) = new_level_spawn_list("spawn/unpack_level")?;
  let dest: PathBuf = path.with_file_name("unpacked");

  let error: CommandError = run(
    &UnpackCommand::<AnySpawnFile>::default(),
    &path,
    &["--dest", &dest.display().to_string(), "--force"],
  )
  .expect_err("a level's spawn list is not unpacked");

  assert_eq!(error.exit_code(), 1);
  assert!(
    error.message().contains("level's spawn list has no unpacked form"),
    "{}",
    error.message()
  );
  assert!(!dest.exists());

  Ok(())
}
