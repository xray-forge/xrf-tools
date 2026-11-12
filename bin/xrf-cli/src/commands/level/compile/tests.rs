//! What `level compile` refuses before running any compiler.

use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::commands::level::compile::command::CompileCommand;
use crate::core::command::command_testing::run_command;

#[test]
fn refuses_a_level_missing_a_steps_input_before_running_it() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("level-compile/missing")?;
  let level: PathBuf = root.join("gamedata").join("levels").join("test_level");
  let sdk: PathBuf = root.join("editors");

  std::fs::create_dir_all(&level)?;
  std::fs::create_dir_all(sdk.join("bin").join("designer").join("release"))?;
  std::fs::write(sdk.join("bin").join("designer").join("release").join("xrAI.exe"), b"")?;
  std::fs::write(level.join("build.prj"), b"")?;

  let error: String = run_command(
    &CompileCommand,
    &[
      String::from("compile"),
      String::from("--path"),
      level.display().to_string(),
      String::from("--steps"),
      String::from("ai"),
      String::from("--sdk"),
      sdk.display().to_string(),
      String::from("--silent"),
    ],
  )
  .unwrap_err()
  .to_string();

  assert!(error.contains("ai step needs build.aimap"), "{error}");

  Ok(())
}

#[test]
fn refuses_a_step_it_does_not_know() {
  let error: String = run_command(
    &CompileCommand,
    &[
      String::from("compile"),
      String::from("--path"),
      String::from("levels/test_level"),
      String::from("--steps"),
      String::from("light"),
      String::from("--sdk"),
      String::from("editors"),
      String::from("--silent"),
    ],
  )
  .unwrap_err()
  .to_string();

  assert!(error.contains("light"), "{error}");
}
