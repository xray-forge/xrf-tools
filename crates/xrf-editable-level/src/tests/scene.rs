use std::fs;
use std::path::{Path, PathBuf};

use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::tests::fixtures::{GUID, binary_part_bytes, chunk, guid_bytes, level_text, spawn_part_text};
use crate::{EditableLevelGuid, EditableLevelPart, EditableLevelScene};

/// A scene on disk: the `.level`, a spawn part, a group saved over two files, an AI map part, and a `.~part` copy of
/// an earlier save, which is no part of the scene.
fn write_scene(root: &Path, part_guid: &EditableLevelGuid) -> XrfResult<PathBuf> {
  let level: PathBuf = root.join("labx8.level");
  let parts: PathBuf = root.join("labx8");

  fs::create_dir_all(&parts)?;
  fs::write(&level, level_text(&GUID)?)?;
  fs::write(parts.join("spawn.part"), spawn_part_text(part_guid, 2)?)?;
  fs::write(parts.join("group.part"), spawn_part_text(&GUID, 2)?)?;
  fs::write(parts.join("group.part1"), spawn_part_text(&GUID, 2)?)?;
  fs::write(parts.join("group.part3"), spawn_part_text(&GUID, 2)?)?;
  fs::write(parts.join("spawn.~part"), "not a part")?;
  fs::write(
    parts.join("ai_map.part"),
    binary_part_bytes(&[chunk(0x7000, &guid_bytes(&GUID)?), chunk(0x8000 + 13, b"nodes")])?,
  )?;

  Ok(level)
}

#[test]
fn reads_a_scene_and_its_parts_as_the_editor_finds_them() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("xrf-editable-level/scene/read")?;
  let scene: EditableLevelScene = EditableLevelScene::read_from_path::<XRayByteOrder, _>(&write_scene(&root, &GUID)?)?;
  let names: Vec<&str> = scene.tools.iter().map(|tool| tool.name.as_str()).collect();

  assert_eq!(names, vec!["ai_map", "group", "spawn"]);
  assert_eq!(
    scene.get_tool("group").map(|tool| tool.parts.len()),
    Some(2),
    "the third group file follows a gap the editor stops at"
  );

  Ok(())
}

#[test]
fn a_scene_written_to_a_new_place_reads_back_the_same() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("xrf-editable-level/scene/write")?;
  let scene: EditableLevelScene = EditableLevelScene::read_from_path::<XRayByteOrder, _>(&write_scene(&root, &GUID)?)?;
  let copy: PathBuf = root.join("copy").join("labx8.level");

  scene.write_to_path::<XRayByteOrder, _>(&copy)?;

  assert_eq!(EditableLevelScene::read_from_path::<XRayByteOrder, _>(&copy)?, scene);
  assert_eq!(
    fs::read(root.join("copy/labx8/spawn.part"))?,
    fs::read(root.join("labx8/spawn.part"))?
  );
  assert!(root.join("copy/labx8/group.part1").is_file());

  Ok(())
}

#[test]
fn refuses_to_write_beside_a_part_the_scene_would_not_write() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("xrf-editable-level/scene/stale-part")?;
  let level: PathBuf = write_scene(&root, &GUID)?;
  let scene: EditableLevelScene = EditableLevelScene::read_from_path::<XRayByteOrder, _>(&level)?;

  // `group.part3`, past the gap the read stopped at, would be read as the scene's if a `group.part2` were written.
  assert!(scene.write_to_path::<XRayByteOrder, _>(&level).is_err());

  fs::remove_file(root.join("labx8/group.part3"))?;
  scene.write_to_path::<XRayByteOrder, _>(&level)?;

  assert_eq!(EditableLevelScene::read_from_path::<XRayByteOrder, _>(&level)?, scene);

  Ok(())
}

/// Windows and the editor compare names without case, so a part held as `SPAWN.part` is the one `spawn.part` replaces.
#[test]
fn writes_over_a_part_held_in_another_case() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("xrf-editable-level/scene/part-case")?;
  let level: PathBuf = write_scene(&root, &GUID)?;
  let scene: EditableLevelScene = EditableLevelScene::read_from_path::<XRayByteOrder, _>(&level)?;

  fs::remove_file(root.join("labx8/group.part3"))?;
  fs::rename(root.join("labx8/spawn.part"), root.join("labx8/SPAWN.part"))?;

  scene.check_writable(&level)?;

  Ok(())
}

/// A tool's name names its files, so one that is not a single file name would write outside the part directory, and
/// two that differ only in case would write the same file.
#[test]
fn refuses_a_tool_name_that_is_not_one_file_name() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("xrf-editable-level/scene/tool-name")?;
  let level: PathBuf = write_scene(&root, &GUID)?;
  let scene: EditableLevelScene = EditableLevelScene::read_from_path::<XRayByteOrder, _>(&level)?;
  let copy: PathBuf = root.join("copy").join("labx8.level");

  for name in ["..\\outside", "../outside", "c:outside", "nested\\spawn", "..", ""] {
    let mut renamed: EditableLevelScene = scene.clone();

    renamed.tools[0].name = String::from(name);

    assert!(renamed.get_part_paths(&copy).is_err(), "'{name}' is refused");
    assert!(
      renamed.write_to_path::<XRayByteOrder, _>(&copy).is_err(),
      "'{name}' is refused"
    );
  }

  let mut doubled: EditableLevelScene = scene;

  doubled.tools[0].name = String::from("SPAWN");
  doubled.tools[1].name = String::from("spawn");

  assert!(doubled.write_to_path::<XRayByteOrder, _>(&copy).is_err());
  assert!(!copy.exists(), "nothing is written");

  Ok(())
}

#[test]
fn names_the_files_the_editor_reads_as_parts() {
  for (file_name, is_part) in [
    ("spawn.part", true),
    ("group.part1", true),
    ("group.part12", true),
    ("spawn.~part", false),
    (".part", false),
    ("spawn.partx", false),
    ("spawn.level", false),
  ] {
    assert_eq!(EditableLevelPart::is_part_file_name(file_name), is_part, "{file_name}");
  }
}

#[test]
fn refuses_a_part_saved_for_another_scene() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("xrf-editable-level/scene/stale")?;
  let stale: EditableLevelGuid = EditableLevelGuid { g0: 1, g1: 2 };

  assert!(EditableLevelScene::read_from_path::<XRayByteOrder, _>(&write_scene(&root, &stale)?).is_err());

  Ok(())
}
