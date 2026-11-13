//! The command lines the SDK's compilers are started with.

use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::LevelCompileStep;
use crate::sdk::level_sdk_program::LevelSdkProgram;

/// An SDK `editors` directory whose compilers are empty files.
fn sdk() -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "sdk"))?;

  std::fs::create_dir_all(root.join("bin/compilers"))?;
  std::fs::write(root.join("bin/compilers/xrLC.exe"), b"")?;

  Ok(root)
}

fn arguments(sdk: &Path, step: LevelCompileStep) -> XrfResult<Option<(Vec<String>, bool)>> {
  Ok(LevelSdkProgram::of(sdk, step)?.map(|program| (program.get_arguments("mp_darkvalley"), program.is_silent())))
}

#[test]
fn runs_xrlc_silent() -> XrfResult {
  assert_eq!(
    arguments(&sdk()?, LevelCompileStep::Geometry)?,
    Some((
      vec![
        String::from("-f"),
        String::from("mp_darkvalley"),
        String::from("-silent")
      ],
      true
    ))
  );

  Ok(())
}

#[test]
fn runs_an_sdk_program_for_exactly_the_steps_the_sdk_runs() -> XrfResult {
  let sdk: PathBuf = sdk()?;

  for step in LevelCompileStep::ALL {
    assert_eq!(LevelSdkProgram::of(&sdk, step)?.is_some(), step.is_sdk(), "{step:?}");
  }

  assert_eq!(
    LevelCompileStep::ALL.map(LevelCompileStep::is_sdk),
    [true, false, false],
    "the SDK runs the geometry alone"
  );

  Ok(())
}
