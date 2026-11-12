//! What a compile runs, and what stops it before anything runs.

use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_job::JobHandle;
use xrf_test_utils::GeneratedResources;

use crate::{LevelCompileRequest, LevelCompileStep, LevelCompiler};

/// An SDK `editors` directory whose compilers are empty files: enough to be found, never run by these tests.
fn sdk(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), name))?;

  for (directory, program) in [("bin/compilers", "xrLC.exe"), ("bin/designer/release", "xrAI.exe")] {
    std::fs::create_dir_all(root.join(directory))?;
    std::fs::write(root.join(directory).join(program), b"")?;
  }

  Ok(root)
}

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

fn request(level: &Path, sdk: &Path, steps: &[LevelCompileStep]) -> LevelCompileRequest {
  LevelCompileRequest {
    level: level.to_path_buf(),
    steps: steps.to_vec(),
    sdk: Some(sdk.to_path_buf()),
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
  assert_eq!(LevelCompileStep::from_name("details"), Some(LevelCompileStep::Details));
  assert_eq!(LevelCompileStep::from_name("light"), None);
}

#[test]
fn refuses_a_step_whose_input_is_missing_before_running_anything() -> XrfResult {
  let level: PathBuf = level("missing_input", &["build.prj", "build.details"])?;
  let error: String = LevelCompiler::compile(
    &request(&level, &sdk("missing_input_sdk")?, &[LevelCompileStep::Details]),
    &JobHandle::inert(),
  )
  .unwrap_err()
  .to_string();

  assert!(error.contains("details step needs build.cform"), "{error}");

  Ok(())
}

#[test]
fn takes_what_the_geometry_writes_as_given_when_it_runs_first() -> XrfResult {
  // Without the geometry, the AI map needs xrLC's build.cform; with it, the check moves on to the textures, which an
  // empty build.prj cannot name, and so fails reading it rather than on a missing input.
  let level: PathBuf = level("chained", &["build.prj", "build.aimap"])?;
  let sdk: PathBuf = sdk("chained_sdk")?;
  let alone: String = LevelCompiler::compile(&request(&level, &sdk, &[LevelCompileStep::Ai]), &JobHandle::inert())
    .unwrap_err()
    .to_string();
  let chained: String = LevelCompiler::compile(
    &request(&level, &sdk, &[LevelCompileStep::Geometry, LevelCompileStep::Ai]),
    &JobHandle::inert(),
  )
  .unwrap_err()
  .to_string();

  assert!(alone.contains("ai step needs build.cform"), "{alone}");
  assert!(!chained.contains("needs build.cform"), "{chained}");

  Ok(())
}

#[test]
fn refuses_an_sdk_without_the_steps_program() -> XrfResult {
  let level: PathBuf = level("no_sdk", &["build.prj"])?;
  let empty: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "no_sdk_editors"))?;
  let error: String = LevelCompiler::compile(
    &request(&level, &empty, &[LevelCompileStep::Geometry]),
    &JobHandle::inert(),
  )
  .unwrap_err()
  .to_string();

  assert!(error.contains("xrLC.exe"), "{error}");

  Ok(())
}

#[test]
fn refuses_a_step_the_sdk_runs_without_the_sdk() -> XrfResult {
  let level: PathBuf = level("sdk_unnamed", &["build.prj", "build.aimap", "build.cform"])?;
  let error: String = LevelCompiler::compile(
    &LevelCompileRequest {
      sdk: None,
      ..request(&level, Path::new(""), &[LevelCompileStep::Ai])
    },
    &JobHandle::inert(),
  )
  .unwrap_err()
  .to_string();

  assert!(error.contains("ai step runs the SDK's compiler"), "{error}");

  Ok(())
}

#[test]
fn refuses_the_sdk_more_than_one_textures_root() -> XrfResult {
  let level: PathBuf = level("two_roots", &["build.prj"])?;
  let error: String = LevelCompiler::compile(
    &LevelCompileRequest {
      textures: vec![level.join("first"), level.join("second")],
      ..request(&level, &sdk("two_roots_sdk")?, &[LevelCompileStep::Geometry])
    },
    &JobHandle::inert(),
  )
  .unwrap_err()
  .to_string();

  assert!(error.contains("one textures root, not 2"), "{error}");

  Ok(())
}
