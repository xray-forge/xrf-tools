use std::fs;
use std::path::{Path, PathBuf};

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::EditableSpawnPointObject;
use xrf_error::XrfResult;
use xrf_level::{BuildPrjFile, LevelEnvModFile, LevelEnvModifier, LevelPsStaticFile};
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
fn needs_the_game_materials_beside_the_objects_to_write_build_prj() -> XrfResult {
  let root: PathBuf = scene_root("geometry-inputs")?;
  let built: PathBuf = root.join("built");
  let objects: String = root.join("objects").display().to_string();

  assert!(
    run(
      "build",
      &root.join("scenes/test_level.level"),
      &built,
      &["--objects", &objects]
    )
    .is_err()
  );
  assert!(!built.join("level.game").exists(), "a refusal writes nothing");
  assert!(!built.join(BuildPrjFile::FILE_NAME).exists());

  Ok(())
}

#[test]
fn needs_objects_named_or_beside_the_scene_to_write_build_prj() -> XrfResult {
  let root: PathBuf = scene_root("objects-default")?;
  let built: PathBuf = root.join("built");
  let game_materials: String = root.join("gamemtl.xr").display().to_string();
  let error: String = get_error(run(
    "build",
    &root.join("scenes/test_level.level"),
    &built,
    &["--game-materials", &game_materials],
  ));

  assert!(
    error.contains("objects directory lies beside the scene's"),
    "Expect the refusal to name the objects it looked for: {error}"
  );
  assert!(!built.join("level.game").exists(), "a refusal writes nothing");

  Ok(())
}

#[test]
fn refuses_to_replace_a_levels_files_without_force() -> XrfResult {
  let root: PathBuf = scene_root("existing")?;
  let level: PathBuf = root.join("level");
  let before: Vec<u8> = fs::read(level.join(LevelEnvModFile::FILE_NAME))?;
  let error: String = get_error(run("build", &root.join("scenes/test_level.level"), &level, &[]));

  assert!(
    error.contains("already holds") && error.contains(LevelEnvModFile::FILE_NAME),
    "Expect the refusal to name the existing file: {error}"
  );
  assert!(!level.join("level.game").exists(), "a refusal writes nothing");

  run("build", &root.join("scenes/test_level.level"), &level, &["--force"]).expect("the scene builds over");

  assert_eq!(fs::read(level.join(LevelEnvModFile::FILE_NAME))?, before);

  Ok(())
}

#[test]
fn refuses_a_file_of_its_own_unforced_and_removes_it_forced_where_the_scene_no_longer_builds_it() -> XrfResult {
  let root: PathBuf = scene_root("stale")?;
  let built: PathBuf = root.join("built");
  let scene: PathBuf = root.join("scenes/test_level.level");

  fs::create_dir_all(&built)?;
  // Named in another case than the build writes, which the game reads as the same file.
  fs::write(built.join("Level.PS_Static"), b"particles")?;

  let error: String = get_error(run("build", &scene, &built, &[]));

  assert!(
    error.contains("Level.PS_Static"),
    "Expect the refusal to name the file: {error}"
  );
  assert!(!built.join("level.game").exists(), "a refusal writes nothing");

  run("build", &scene, &built, &["--force"]).expect("the forced build writes");

  assert!(built.join("level.game").is_file());
  assert!(
    !built.join("Level.PS_Static").exists() && !built.join(LevelPsStaticFile::FILE_NAME).exists(),
    "a file of the build's own it no longer writes is removed"
  );

  Ok(())
}
