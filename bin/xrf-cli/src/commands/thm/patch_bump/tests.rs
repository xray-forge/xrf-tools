//! What `thm patch-bump` does to a descriptor's bump declaration, what it keeps, and what it refuses.

use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_chunk::XRayByteOrder;
use xrf_test_utils::GeneratedResources;
use xrf_thm::{ThmBumpChunk, ThmBumpMode, ThmFile, ThmTextureParamChunk};

use crate::commands::thm::patch_bump::command::PatchBumpCommand;
use crate::commands::thm::tests::fixtures::{raw_chunk, sound_thumbnail, texture_descriptor};
use crate::core::command::CommandResult;
use crate::core::command::command_testing::run_command_for_result;

fn descriptor(case: &str, bump: ThmBumpChunk) -> CommandResult<PathBuf> {
  let path: PathBuf = GeneratedResources::path(&format!("thm_patch_bump/{case}.thm"));

  std::fs::create_dir_all(path.parent().expect("case directory"))?;
  std::fs::write(
    &path,
    ThmFile {
      bump: Some(bump),
      ..texture_descriptor()
    }
    .write_to_bytes::<XRayByteOrder>()?,
  )?;

  Ok(path)
}

fn used(name: &str, mode: ThmBumpMode) -> ThmBumpChunk {
  ThmBumpChunk {
    mode,
    name: name.to_owned(),
    ..ThmBumpChunk::default()
  }
}

fn run(path: &Path, arguments: &[&str]) -> CommandResult<Value> {
  let mut command: Vec<String> = vec![String::from("patch-bump"), path.display().to_string()];

  command.extend(arguments.iter().map(|it| (*it).to_owned()));
  command.push(String::from("--json"));

  Ok(run_command_for_result(&PatchBumpCommand, &command)?.expect("Expect a reported result"))
}

fn bump_of(path: &Path) -> CommandResult<Option<ThmBumpChunk>> {
  Ok(ThmFile::read_from_path::<XRayByteOrder, _>(&path)?.bump)
}

#[test]
fn repoints_the_name_and_keeps_the_mode_and_every_other_field() -> CommandResult {
  let path: PathBuf = descriptor("repoint", used("wpn\\source_bump", ThmBumpMode::UseParallax))?;
  let param: Option<ThmTextureParamChunk> = ThmFile::read_from_path::<XRayByteOrder, _>(&path)?.texture_param;
  let report: Value = run(&path, &["--to", "wpn\\renamed_bump"])?;

  assert_eq!(
    bump_of(&path)?,
    Some(used("wpn\\renamed_bump", ThmBumpMode::UseParallax))
  );
  assert_eq!(ThmFile::read_from_path::<XRayByteOrder, _>(&path)?.texture_param, param);
  assert_eq!(report["previousName"], "wpn\\source_bump");
  assert_eq!(report["previousMode"], 3);
  assert_eq!(report["isDryRun"], false);

  Ok(())
}

#[test]
fn turns_a_declaration_off_as_the_sdk_writes_one() -> CommandResult {
  let path: PathBuf = descriptor("off", used("tile\\tile_walls_red_01_bump", ThmBumpMode::Use))?;

  run(&path, &["--off"])?;

  assert_eq!(bump_of(&path)?, Some(used("", ThmBumpMode::None)));

  Ok(())
}

#[test]
fn writes_a_destination_and_nothing_on_a_dry_run() -> CommandResult {
  let path: PathBuf = descriptor("destination", used("wpn\\source_bump", ThmBumpMode::Use))?;
  let original: Vec<u8> = std::fs::read(&path)?;
  let destination: PathBuf = path.with_file_name("destination_patched.thm");
  let _ = std::fs::remove_file(&destination);

  let report: Value = run(&path, &["--to", "wpn\\other_bump", "--dry-run"])?;

  assert_eq!(report["isDryRun"], true);
  assert_eq!(
    report["patchedSize"],
    original.len() - 1,
    "one character shorter a name"
  );
  assert_eq!(std::fs::read(&path)?, original);

  run(
    &path,
    &["--to", "wpn\\other_bump", "-d", &destination.display().to_string()],
  )?;

  assert_eq!(std::fs::read(&path)?, original);
  assert_eq!(bump_of(&destination)?, Some(used("wpn\\other_bump", ThmBumpMode::Use)));

  Ok(())
}

#[test]
fn refuses_a_file_declaring_no_bump() -> CommandResult {
  let path: PathBuf = GeneratedResources::path("thm_patch_bump/sound.thm");

  std::fs::create_dir_all(path.parent().expect("case directory"))?;
  std::fs::write(&path, sound_thumbnail().write_to_bytes::<XRayByteOrder>()?)?;

  assert!(run(&path, &["--to", "any"]).is_err());

  Ok(())
}

#[test]
fn refuses_a_descriptor_it_could_not_keep_byte_for_byte() -> CommandResult {
  let path: PathBuf = descriptor("unkept", used("wpn\\source_bump", ThmBumpMode::Use))?;
  let original: Vec<u8> = std::fs::read(&path)?;
  // An unknown chunk after the ten-byte version chunk, which the writer would move to the end of the file.
  let reordered: Vec<u8> = [&original[..10], &raw_chunk(0x0900, &[1, 2, 3]), &original[10..]].concat();

  std::fs::write(&path, &reordered)?;

  let refusal: String = run(&path, &["--to", "wpn\\other_bump"])
    .expect_err("an unkept descriptor is refused")
    .to_string();

  assert!(refusal.contains("would not be written back byte for byte"), "{refusal}");
  assert_eq!(std::fs::read(&path)?, reordered);

  Ok(())
}
