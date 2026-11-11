//! What `level decompile` writes, what it refuses, and that a refusal writes nothing.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::EditableSpawnPointObject;
use xrf_error::XrfResult;
use xrf_level::{LevelEnvModFile, LevelEnvModifier};
use xrf_math::Vector3d;
use xrf_rawdata::LevelSceneProvenance;
use xrf_test_utils::GeneratedResources;

use crate::commands::level::decompile::command::DecompileCommand;
use crate::core::command::CommandResult;
use crate::core::command::command_testing::run_command;

fn decompile(source: &Path, destination: &Path, extra: &[&str]) -> CommandResult {
  let mut arguments: Vec<String> = vec![
    String::from("decompile"),
    String::from("--path"),
    source.display().to_string(),
    String::from("--dest"),
    destination.display().to_string(),
    String::from("--silent"),
  ];

  arguments.extend(extra.iter().map(|it| String::from(*it)));

  run_command(&DecompileCommand, &arguments)
}

fn get_error(verdict: CommandResult) -> String {
  match verdict {
    Ok(()) => panic!("Expect the decompile to be refused"),
    Err(error) => error.to_string(),
  }
}

/// A test's root holding `level/`, a compiled level of one environment modifier, and `scenes/`, empty.
fn level_root(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&format!("level-decompile/{name}"))?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  LevelEnvModFile {
    version: EditableSpawnPointObject::VERSION,
    modifiers: vec![LevelEnvModifier {
      position: Vector3d::new(1.0, 2.0, 3.0),
      radius: 10.0,
      power: 1.0,
      far_plane: 300.0,
      fog_color: Vector3d::ZERO,
      fog_density: 0.5,
      ambient: Vector3d::ZERO,
      sky_color: Vector3d::ZERO,
      hemi_color: Vector3d::ZERO,
      use_flags: Some(63),
    }],
  }
  .write::<XRayByteOrder>(&mut writer)?;
  fs::create_dir_all(root.join("level"))?;
  fs::create_dir_all(root.join("scenes"))?;
  fs::write(root.join("level").join(LevelEnvModFile::FILE_NAME), writer.buffer)?;

  Ok(root)
}

#[test]
fn writes_the_scene_its_parts_and_its_sidecar() -> XrfResult {
  let root: PathBuf = level_root("written")?;
  let destination: PathBuf = root.join("scenes").join("test_level.level");

  decompile(&root.join("level"), &destination, &[]).expect("the level decompiles");

  let sidecar: LevelSceneProvenance =
    serde_json::from_slice(&fs::read(root.join("scenes/test_level.level.xrf.json"))?)?;

  assert!(destination.is_file());
  assert!(root.join("scenes/test_level/spawn.part").is_file());
  assert_eq!(sidecar.level, "test_level");
  assert_eq!(sidecar.tool, "xrf-cli level decompile");

  Ok(())
}

#[test]
fn refuses_to_replace_a_scene_without_force() -> XrfResult {
  let root: PathBuf = level_root("existing")?;
  let destination: PathBuf = root.join("scenes").join("test_level.level");

  fs::write(&destination, b"authored")?;

  let error: String = get_error(decompile(&root.join("level"), &destination, &[]));

  assert!(
    error.contains("--force"),
    "Expect the refusal to name the flag: {error}"
  );
  assert_eq!(fs::read(&destination)?, b"authored");
  assert!(!root.join("scenes/test_level.level.xrf.json").exists());

  Ok(())
}

#[test]
fn refuses_with_force_a_scene_whose_parts_it_would_not_all_replace() -> XrfResult {
  // An authored scene's part the decompile does not write would stay and be read as part of the decompiled scene, so
  // even --force writes nothing, the sidecar included.
  let root: PathBuf = level_root("authored")?;
  let destination: PathBuf = root.join("scenes").join("test_level.level");

  fs::create_dir_all(root.join("scenes/test_level"))?;
  fs::write(root.join("scenes/test_level/scene_object.part"), b"[guid]")?;

  let error: String = get_error(decompile(&root.join("level"), &destination, &["--force"]));

  assert!(
    error.contains("scene_object.part"),
    "Expect the refusal to name the part: {error}"
  );
  assert!(!destination.exists());
  assert!(!root.join("scenes/test_level.level.xrf.json").exists());

  Ok(())
}

/// With no `spawns/all.spawn` beside the levels, the entities come from the level's own lists and no root is recorded.
#[test]
fn reads_no_root_spawn_where_the_default_one_is_absent() -> XrfResult {
  let root: PathBuf = level_root("no_root")?;
  let destination: PathBuf = root.join("scenes").join("test_level.level");

  decompile(&root.join("level"), &destination, &[]).expect("the level decompiles");

  let sidecar: LevelSceneProvenance =
    serde_json::from_slice(&fs::read(root.join("scenes/test_level.level.xrf.json"))?)?;

  assert_eq!(sidecar.root_spawn, None);

  Ok(())
}

/// A root spawn asked for by name must exist, so a mistyped name is not taken for a level of no root spawn.
#[test]
fn refuses_a_named_root_spawn_that_does_not_exist() -> XrfResult {
  let root: PathBuf = level_root("missing_root")?;
  let destination: PathBuf = root.join("scenes").join("test_level.level");
  let error: String = get_error(decompile(
    &root.join("level"),
    &destination,
    &["--spawn", "test_missing"],
  ));

  assert!(
    error.contains("test_missing.spawn"),
    "Expect the refusal to name the file: {error}"
  );
  assert!(!destination.exists());

  Ok(())
}

#[test]
fn puts_objects_beside_the_levels_directory_by_default() {
  let objects_of = |destination: &str| DecompileCommand::get_default_objects_root(Path::new(destination));

  assert_eq!(
    objects_of("rawdata/levels/zaton.level"),
    Path::new("rawdata").join("objects"),
    "the SDK's rawdata/objects beside rawdata/levels"
  );
  assert_eq!(
    objects_of("levels/zaton.level"),
    Path::new("levels").join("..").join("objects")
  );
  assert_eq!(objects_of("zaton.level"), Path::new("..").join("objects"));
}

#[test]
fn puts_textures_in_the_gamedata_beside_the_rawdata_by_default() {
  assert_eq!(
    DecompileCommand::get_default_textures_root(Path::new("editors/rawdata/levels/zaton.level")),
    Path::new("editors/rawdata")
      .join("..")
      .join("gamedata")
      .join("textures"),
    "the SDK's editors/gamedata/textures beside editors/rawdata"
  );
}

#[test]
fn refuses_a_named_library_that_does_not_exist() -> XrfResult {
  let root: PathBuf = level_root("library")?;
  let error: String = get_error(decompile(
    &root.join("level"),
    &root.join("test_level.level"),
    &["--game-materials", &root.join("missing.xr").to_string_lossy()],
  ));

  assert!(
    error.contains("does not exist"),
    "Expect the refusal to name the missing library: {error}"
  );

  Ok(())
}

#[test]
fn refuses_a_destination_that_is_not_a_level() -> XrfResult {
  let root: PathBuf = level_root("named")?;
  let error: String = get_error(decompile(
    &root.join("level"),
    &root.join("scenes").join("test_level"),
    &[],
  ));

  assert!(
    error.contains(".level"),
    "Expect the refusal to name the extension: {error}"
  );

  Ok(())
}
