//! What `thm info` reports for each kind of thumbnail, and what it refuses.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;
use xrf_test_utils::GeneratedResources;
use xrf_thm::{ThmFile, ThmGroupFile, ThmObjectFile, ThmSoundFile};

use crate::commands::thm::info::command::InfoCommand;
use crate::commands::thm::tests::fixtures::{
  group_thumbnail, object_thumbnail, raw_chunk, sound_thumbnail, texture_descriptor,
};
use crate::core::command::CommandResult;
use crate::core::command::command_testing::run_command_for_result;

fn file(name: &str, bytes: &[u8]) -> CommandResult<PathBuf> {
  let path: PathBuf = GeneratedResources::path(&format!("thm_info/{name}"));

  std::fs::create_dir_all(path.parent().expect("case directory"))?;
  std::fs::write(&path, bytes)?;

  Ok(path)
}

fn run(path: &Path) -> CommandResult<Value> {
  let command: Vec<String> = vec![String::from("info"), path.display().to_string(), String::from("--json")];

  Ok(run_command_for_result(&InfoCommand, &command)?.expect("Expect a reported result"))
}

fn assert_reported<T: Serialize>(name: &str, bytes: &[u8], kind: &str, expected: &T) -> CommandResult {
  let report: Value = run(&file(name, bytes)?)?;

  assert_eq!(report["kind"], kind, "{name}");
  assert_eq!(report["file"], serde_json::to_value(expected)?, "{name}");

  Ok(())
}

#[test]
fn reports_a_texture_descriptor_as_the_reader_reads_it() -> CommandResult {
  let descriptor: ThmFile = texture_descriptor();

  assert_reported("texture.thm", &descriptor.write_to_bytes()?, "texture", &descriptor)
}

#[test]
fn reports_an_object_thumbnail_with_its_size() -> CommandResult {
  let object: ThmObjectFile = object_thumbnail()?;

  assert_reported("object.thm", &object.write_to_bytes()?, "object", &object)
}

#[test]
fn tells_a_group_thumbnail_from_the_object_type_it_is_stamped_with() -> CommandResult {
  let group: ThmGroupFile = group_thumbnail()?;

  assert_reported("group.thm", &group.write_to_bytes()?, "group", &group)
}

#[test]
fn reports_a_sound_thumbnail_with_its_parameters() -> CommandResult {
  let sound: ThmSoundFile = sound_thumbnail();

  assert_reported("sound.thm", &sound.write_to_bytes()?, "sound", &sound)
}

#[test]
fn refuses_a_file_no_thumbnail_reader_accepts() -> CommandResult {
  let unknown: Vec<u8> = raw_chunk(ThmFile::VERSION_CHUNK_ID, &0x7777_u16.to_le_bytes());

  assert!(run(&file("unknown.thm", &unknown)?).is_err());

  Ok(())
}
