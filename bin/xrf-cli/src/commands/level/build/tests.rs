//! What `level build` writes from a scene, what it refuses, and that a refusal writes nothing.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::EditableSpawnPointObject;
use xrf_error::XrfResult;
use xrf_level::{LevelEnvModFile, LevelEnvModifier, LevelPsStaticFile};
use xrf_math::Vector3d;
use xrf_test_utils::GeneratedResources;

use crate::commands::level::build::command::BuildCommand;
use crate::commands::level::decompile::command::DecompileCommand;
use crate::core::command::CommandResult;
use crate::core::command::command_testing::run_command;

fn run(command: &str, source: &Path, destination: &Path, extra: &[&str]) -> CommandResult {
  let mut arguments: Vec<String> = vec![
    String::from(command),
    String::from("--path"),
    source.display().to_string(),
    String::from("--dest"),
    destination.display().to_string(),
    String::from("--silent"),
  ];

  arguments.extend(extra.iter().map(|it| String::from(*it)));

  match command {
    "build" => run_command(&BuildCommand, &arguments),
    _ => run_command(&DecompileCommand, &arguments),
  }
}

fn get_error(verdict: CommandResult) -> String {
  match verdict {
    Ok(()) => panic!("Expect the build to be refused"),
    Err(error) => error.to_string(),
  }
}

/// A test's root holding `level/`, a compiled level of one environment modifier, and `scenes/test_level.level`, the
/// scene decompiled from it.
fn scene_root(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&format!("level-build/{name}"))?;
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
  run(
    "decompile",
    &root.join("level"),
    &root.join("scenes/test_level.level"),
    &[],
  )
  .expect("the level decompiles");

  Ok(root)
}

#[test]
fn builds_the_game_files_every_build_writes() -> XrfResult {
  let root: PathBuf = scene_root("written")?;
  let built: PathBuf = root.join("built");

  run("build", &root.join("scenes/test_level.level"), &built, &[]).expect("the scene builds");

  assert_eq!(
    fs::read(built.join(LevelEnvModFile::FILE_NAME))?,
    fs::read(root.join("level").join(LevelEnvModFile::FILE_NAME))?
  );
  assert!(built.join("level.game").is_file());
  assert!(built.join("level.fog_vol").is_file());
  assert!(!built.join(LevelPsStaticFile::FILE_NAME).exists());

  Ok(())
}

#[test]
fn refuses_to_replace_a_levels_files_without_force() -> XrfResult {
  let root: PathBuf = scene_root("existing")?;
  let level: PathBuf = root.join("level");
  let before: Vec<u8> = fs::read(level.join(LevelEnvModFile::FILE_NAME))?;
  let error: String = get_error(run("build", &root.join("scenes/test_level.level"), &level, &[]));

  assert!(
    error.contains("--force"),
    "Expect the refusal to name the flag: {error}"
  );
  assert!(!level.join("level.game").exists(), "a refusal writes nothing");

  run("build", &root.join("scenes/test_level.level"), &level, &["--force"]).expect("the scene builds over");

  assert_eq!(fs::read(level.join(LevelEnvModFile::FILE_NAME))?, before);

  Ok(())
}

#[test]
fn refuses_a_level_holding_a_file_the_scene_does_not_build() -> XrfResult {
  let root: PathBuf = scene_root("stale")?;
  let built: PathBuf = root.join("built");

  fs::create_dir_all(&built)?;
  fs::write(built.join(LevelPsStaticFile::FILE_NAME), b"particles")?;

  let error: String = get_error(run(
    "build",
    &root.join("scenes/test_level.level"),
    &built,
    &["--force"],
  ));

  assert!(
    error.contains("level.ps_static"),
    "Expect the refusal to name the file: {error}"
  );
  assert!(!built.join("level.game").exists(), "a refusal writes nothing");
  assert_eq!(fs::read(built.join(LevelPsStaticFile::FILE_NAME))?, b"particles");

  Ok(())
}
