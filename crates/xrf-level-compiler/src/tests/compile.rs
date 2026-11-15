//! What a compile runs, and what stops it before anything runs.

use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_job::JobHandle;
use xrf_shaders::ShaderCompilerFile;
use xrf_test_utils::GeneratedResources;

use crate::{AiMapCovers, LevelCompileRequest, LevelCompileStep, LevelCompiler};

/// A level directory holding `files`, empty each.
fn level(name: &str, files: &[&str]) -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), name))?;
  let level: PathBuf = root.join("levels").join("test_level");

  std::fs::create_dir_all(&level)?;

  for file in files {
    std::fs::write(level.join(file), b"")?;
  }

  Ok(level)
}

/// A game data directory holding an empty compiler shader library: enough for the checks to find it, never read by
/// these tests.
fn gamedata(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), name))?;

  std::fs::write(root.join(ShaderCompilerFile::FILE_NAME), b"")?;

  Ok(root)
}

fn request(level: &Path, steps: &[LevelCompileStep]) -> LevelCompileRequest {
  LevelCompileRequest {
    level: level.to_path_buf(),
    steps: steps.to_vec(),
    textures: vec![level.join("textures")],
    ..LevelCompileRequest::default()
  }
}

#[test]
fn runs_each_step_once_in_its_order_and_every_step_when_none_is_named() {
  assert_eq!(
    LevelCompiler::get_steps(&[LevelCompileStep::Ai, LevelCompileStep::Geometry, LevelCompileStep::Ai]),
    [LevelCompileStep::Geometry, LevelCompileStep::Ai]
  );
  assert_eq!(LevelCompiler::get_steps(&[]), LevelCompileStep::ALL);
  assert_eq!(LevelCompileStep::from_name("light"), Some(LevelCompileStep::Light));
  assert_eq!(LevelCompileStep::from_name("lights"), None);
}

#[test]
fn refuses_the_light_step_without_the_geometry_step_before_running_anything() -> XrfResult {
  let level: PathBuf = level("light_alone", &["build.prj"])?;
  let error: String = LevelCompiler::compile(
    &request(&level, &[LevelCompileStep::Light, LevelCompileStep::Details]),
    &JobHandle::inert(),
  )
  .unwrap_err()
  .to_string();

  // The light is baked in the geometry's pass, so it has nothing to light alone.
  assert!(error.contains("light step runs only with the geometry step"), "{error}");

  Ok(())
}

#[test]
fn refuses_a_step_whose_input_is_missing_before_running_anything() -> XrfResult {
  let level: PathBuf = level("missing_input", &["build.prj", "build.details"])?;
  let error: String = LevelCompiler::compile(&request(&level, &[LevelCompileStep::Details]), &JobHandle::inert())
    .unwrap_err()
    .to_string();

  assert!(error.contains("details step needs build.cform"), "{error}");

  Ok(())
}

#[test]
fn takes_what_the_geometry_writes_as_given_when_it_runs_first() -> XrfResult {
  // Without the geometry, the AI map needs the geometry's build.cform; with it, the checks move on past the library to
  // the textures, which an empty build.prj cannot name, and so fail reading it rather than on a missing input.
  let level: PathBuf = level("chained", &["build.prj", "build.aimap"])?;
  let gamedata: PathBuf = gamedata("chained_gamedata")?;
  let compile = |steps: &[LevelCompileStep]| {
    LevelCompiler::compile(
      &LevelCompileRequest {
        gamedata: gamedata.clone(),
        ..request(&level, steps)
      },
      &JobHandle::inert(),
    )
    .unwrap_err()
    .to_string()
  };
  let alone: String = compile(&[LevelCompileStep::Ai]);
  let chained: String = compile(&[LevelCompileStep::Geometry, LevelCompileStep::Ai]);

  assert!(alone.contains("ai step needs build.cform"), "{alone}");
  assert!(
    !chained.contains("needs build.cform") && !chained.contains(ShaderCompilerFile::FILE_NAME),
    "{chained}"
  );

  Ok(())
}

#[test]
fn refuses_the_geometry_step_without_the_compiler_shader_library() -> XrfResult {
  let level: PathBuf = level("no_library", &["build.prj"])?;
  let error: String = LevelCompiler::compile(&request(&level, &[LevelCompileStep::Geometry]), &JobHandle::inert())
    .unwrap_err()
    .to_string();

  // Our geometry compiler reads the library for each face's flags; without it the compile stops before anything runs.
  assert!(
    error.contains("geometry step needs") && error.contains("shaders_xrlc.xr"),
    "{error}"
  );

  Ok(())
}

#[test]
fn needs_no_ray_model_for_an_ai_map_with_no_covers() -> XrfResult {
  let level: PathBuf = level("ai_no_covers", &["build.aimap"])?;
  let refuse = |covers: AiMapCovers| {
    LevelCompiler::compile(
      &LevelCompileRequest {
        ai_covers: covers,
        ..request(&level, &[LevelCompileStep::Ai])
      },
      &JobHandle::inert(),
    )
    .unwrap_err()
    .to_string()
  };

  assert!(refuse(AiMapCovers::Full).contains("ai step needs build.prj"));
  // With no covers the checks pass, no library needed, and the empty build.aimap fails to read.
  assert!(!refuse(AiMapCovers::None).contains("needs"));

  Ok(())
}
