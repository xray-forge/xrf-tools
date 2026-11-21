use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_error::XrfResult;
use xrf_spawn::{AnySpawnFile, LevelSpawnFile};
use xrf_test_utils::GeneratedResources;

use crate::commands::packed_file::{PackCommand, ReUnpackCommand, RepackCommand, UnpackCommand, VerifyCommand};
use crate::commands::spawn::fixtures::{
  new_level_object, new_root_spawn, new_typed_level_object, write_level_spawn_list,
};
use crate::core::command::command_testing::{run_command, run_command_for_result};
use crate::core::command::{CommandError, CommandResult, GenericCommand};

fn arguments_of<T: GenericCommand>(command: &T, path: &Path, extra: &[&str]) -> Vec<String> {
  let mut arguments: Vec<String> = vec![
    String::from(command.operation()),
    path.display().to_string(),
    String::from("--silent"),
  ];

  arguments.extend(extra.iter().map(|it| String::from(*it)));

  arguments
}

fn run<T: GenericCommand>(command: &T, path: &Path, extra: &[&str]) -> CommandResult {
  run_command(command, &arguments_of(command, path, extra))
}

fn run_for_result<T: GenericCommand>(command: &T, path: &Path, extra: &[&str]) -> CommandResult<Value> {
  Ok(run_command_for_result(command, &arguments_of(command, path, extra))?.expect("a result was asked for"))
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

#[test]
fn unpacks_a_level_spawn_list_and_packs_it_back_byte_for_byte() -> CommandResult {
  let path: PathBuf = GeneratedResources::create_dir("spawn/unpack_level")?.join("level.spawn");
  let bytes: Vec<u8> = write_level_spawn_list(
    &path,
    vec![
      new_typed_level_object(0, "zat_graph_point")?,
      new_level_object(1, "m_stalker", "esc_wolf"),
      new_level_object(2, "m_stalker", "esc_fox"),
    ],
  )?;
  let unpacked: PathBuf = path.with_file_name("unpacked");
  let re_unpacked: PathBuf = path.with_file_name("re_unpacked");
  let repacked: PathBuf = path.with_file_name("repacked.spawn");

  let report: Value = run_for_result(
    &UnpackCommand::<AnySpawnFile>::default(),
    &path,
    &["--dest", &unpacked.display().to_string(), "--json"],
  )?;

  assert_eq!(report["typedObjects"], 1, "{report}");
  assert_eq!(report["verbatimObjects"][0]["objects"], 2, "{report}");
  // Both packets are filler refused at the version their bytes spell; names stay out of the reason, so they share one.
  assert_eq!(
    report["verbatimObjects"][0]["reason"], "Assertion error: Unexpected version 0 of spawn packet, script flags 1799",
    "{report}"
  );

  run(&VerifyCommand::<AnySpawnFile>::default(), &unpacked, &["--unpacked"]).expect("the folder reads");
  run(
    &ReUnpackCommand::<AnySpawnFile>::default(),
    &unpacked,
    &["--dest", &re_unpacked.display().to_string()],
  )
  .expect("the folder re-unpacks");

  assert_eq!(
    fs::read(re_unpacked.join(LevelSpawnFile::UNPACKED_FILE_NAME))?,
    fs::read(unpacked.join(LevelSpawnFile::UNPACKED_FILE_NAME))?
  );

  run(
    &PackCommand::<AnySpawnFile>::default(),
    &unpacked,
    &["--dest", &repacked.display().to_string()],
  )
  .expect("the folder packs");

  assert_eq!(fs::read(&repacked)?, bytes);

  Ok(())
}

#[test]
fn reports_a_root_spawns_unpack_as_the_conversion_alone() -> CommandResult {
  let root: PathBuf = GeneratedResources::create_dir("spawn/unpack_root")?;

  new_root_spawn().write_to_path(&root.join("all.spawn"))?;

  let report: Value = run_for_result(
    &UnpackCommand::<AnySpawnFile>::default(),
    &root.join("all.spawn"),
    &["--dest", &root.join("unpacked").display().to_string(), "--json"],
  )?;
  let mut keys: Vec<&String> = report.as_object().expect("an object").keys().collect();

  keys.sort();

  assert_eq!(keys, ["destination", "readDuration", "source", "writeDuration"]);

  Ok(())
}

#[test]
fn judges_a_folder_of_neither_kind() -> XrfResult {
  let folder: PathBuf = GeneratedResources::create_dir("spawn/neither_folder")?;
  let verified: CommandError = run(&VerifyCommand::<AnySpawnFile>::default(), &folder, &["--unpacked"])
    .expect_err("a folder of neither kind is judged");
  let packed: CommandError = run(
    &PackCommand::<AnySpawnFile>::default(),
    &folder,
    &["--dest", &folder.with_file_name("neither.spawn").display().to_string()],
  )
  .expect_err("a folder of neither kind is not packed");

  assert_eq!(verified.exit_code(), 3);
  assert_eq!(packed.exit_code(), 1);
  assert!(
    packed.message().contains("header.ltx") && packed.message().contains("level_spawns.ltx"),
    "{}",
    packed.message()
  );

  Ok(())
}
