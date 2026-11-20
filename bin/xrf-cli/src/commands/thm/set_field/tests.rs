//! What `thm set-field` changes in a descriptor, what it leaves alone, and what it refuses.

use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_chunk::XRayByteOrder;
use xrf_test_utils::GeneratedResources;
use xrf_thm::{ThmFile, ThmTextureParamChunk};

use crate::commands::thm::set_field::command::SetFieldCommand;
use crate::commands::thm::tests::fixtures::{
  group_thumbnail, object_thumbnail, raw_chunk, sound_thumbnail, texture_descriptor,
};
use crate::core::command::CommandResult;
use crate::core::command::command_testing::run_command_for_result;

fn case(name: &str) -> CommandResult<PathBuf> {
  let directory: PathBuf = GeneratedResources::path(&format!("thm_set_field/{name}"));
  let _ = std::fs::remove_dir_all(&directory);

  std::fs::create_dir_all(&directory)?;

  Ok(directory)
}

fn descriptor() -> CommandResult<Vec<u8>> {
  Ok(texture_descriptor().write_to_bytes::<XRayByteOrder>()?)
}

fn other_thumbnails() -> CommandResult<[(&'static str, Vec<u8>); 3]> {
  Ok([
    ("crate.thm", object_thumbnail()?.write_to_bytes::<XRayByteOrder>()?),
    ("forest.thm", group_thumbnail()?.write_to_bytes::<XRayByteOrder>()?),
    ("door.thm", sound_thumbnail().write_to_bytes::<XRayByteOrder>()?),
  ])
}

fn run(input: &Path, arguments: &[&str]) -> CommandResult<Value> {
  let mut command: Vec<String> = vec![String::from("set-field"), input.display().to_string()];

  command.extend(arguments.iter().map(|it| (*it).to_owned()));
  command.push(String::from("--json"));

  Ok(run_command_for_result(&SetFieldCommand, &command)?.expect("Expect a reported result"))
}

#[test]
fn changes_the_bytes_of_the_field_named_and_no_others() -> CommandResult {
  let path: PathBuf = case("one_field")?.join("wall.thm");
  let original: Vec<u8> = descriptor()?;

  std::fs::write(&path, &original)?;

  let report: Value = run(&path, &["--fade-amount", "40"])?;
  let edited: Vec<u8> = std::fs::read(&path)?;
  let differing: Vec<usize> = (0..original.len())
    .filter(|index| original[*index] != edited[*index])
    .collect();

  assert_eq!(edited.len(), original.len());
  assert_eq!(
    differing.len(),
    1,
    "only the amount's low byte differs, 0 to 40: {differing:?}"
  );
  assert_eq!(edited[differing[0]], 40);
  assert_eq!(
    ThmFile::read_from_bytes::<XRayByteOrder>(edited)?
      .texture_param
      .map(|param: ThmTextureParamChunk| param.fade_amount),
    Some(40)
  );
  assert_eq!(report["changed"].as_array().map(Vec::len), Some(1));
  assert_eq!(report["unchanged"], 0);

  Ok(())
}

#[test]
fn edits_every_texture_descriptor_below_a_directory_and_skips_every_other_kind() -> CommandResult {
  let directory: PathBuf = case("directory")?;
  let others: [(&str, Vec<u8>); 3] = other_thumbnails()?;

  std::fs::create_dir_all(directory.join("nested"))?;
  std::fs::write(directory.join("a.thm"), descriptor()?)?;
  std::fs::write(directory.join("nested").join("b.thm"), descriptor()?)?;

  for (name, bytes) in &others {
    std::fs::write(directory.join("nested").join(name), bytes)?;
  }

  // `b` already says what the run asks, so only `a` is rewritten.
  run(
    &directory.join("nested").join("b.thm"),
    &["--detail-name", "detail\\stone"],
  )?;

  let report: Value = run(&directory, &["--detail-name", "detail\\stone"])?;

  assert_eq!(report["changed"].as_array().map(Vec::len), Some(1));
  assert!(report["changed"][0].as_str().is_some_and(|it| it.ends_with("a.thm")));
  assert_eq!(report["unchanged"], 1);
  assert_eq!(report["skipped"], 3);
  assert_eq!(
    ThmFile::read_from_path::<XRayByteOrder, _>(&directory.join("a.thm"))?
      .detail
      .map(|detail| detail.name),
    Some(String::from("detail\\stone"))
  );

  for (name, bytes) in &others {
    assert_eq!(&std::fs::read(directory.join("nested").join(name))?, bytes, "{name}");
  }

  Ok(())
}

#[test]
fn writes_nothing_when_one_descriptor_cannot_be_kept_byte_for_byte() -> CommandResult {
  let directory: PathBuf = case("unkept")?;
  let original: Vec<u8> = descriptor()?;
  // An unknown chunk after the ten-byte version chunk, which the writer would move to the end of the file.
  let reordered: Vec<u8> = [&original[..10], &raw_chunk(0x0900, &[1, 2, 3]), &original[10..]].concat();

  std::fs::write(directory.join("a.thm"), &original)?;
  std::fs::write(directory.join("b.thm"), &reordered)?;

  let refusal: String = run(&directory, &["--fade-amount", "40"])
    .expect_err("an unkept descriptor refuses the run")
    .to_string();

  assert!(
    refusal.contains("b.thm would not be written back byte for byte"),
    "{refusal}"
  );
  assert_eq!(std::fs::read(directory.join("a.thm"))?, original);
  assert_eq!(std::fs::read(directory.join("b.thm"))?, reordered);

  Ok(())
}

#[test]
fn refuses_each_other_kind_named_on_its_own_and_leaves_it_as_it_was() -> CommandResult {
  let directory: PathBuf = case("named_other_kind")?;

  for (name, bytes) in other_thumbnails()? {
    let path: PathBuf = directory.join(name);

    std::fs::write(&path, &bytes)?;

    assert!(run(&path, &["--fade-amount", "40"]).is_err(), "{name}");
    assert_eq!(std::fs::read(&path)?, bytes, "{name}");
  }

  Ok(())
}

#[test]
fn needs_a_field_to_set_and_writes_nothing_on_a_dry_run() -> CommandResult {
  let path: PathBuf = case("dry_run")?.join("wall.thm");
  let original: Vec<u8> = descriptor()?;

  std::fs::write(&path, &original)?;

  assert!(run(&path, &[]).is_err());

  let report: Value = run(&path, &["--bump-mode", "none", "--dry-run"])?;

  assert_eq!(report["isDryRun"], true);
  assert_eq!(report["changed"].as_array().map(Vec::len), Some(1));
  assert_eq!(std::fs::read(&path)?, original);

  Ok(())
}
