//! The command lines the SDK's compilers are started with.

use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::LevelCompileStep;
use crate::sdk::level_sdk_program::LevelSdkProgram;

/// An SDK `editors` directory whose compilers are empty files.
fn sdk() -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "sdk"))?;

  for (directory, program) in [("bin/compilers", "xrLC.exe"), ("bin/designer/release", "xrAI.exe")] {
    std::fs::create_dir_all(root.join(directory))?;
    std::fs::write(root.join(directory).join(program), b"")?;
  }

  Ok(root)
}

fn arguments(sdk: &Path, step: LevelCompileStep) -> XrfResult<Option<(Vec<String>, bool)>> {
  Ok(LevelSdkProgram::of(sdk, step)?.map(|program| (program.get_arguments("mp_darkvalley"), program.is_silent())))
}

#[test]
fn skips_the_closing_box_where_the_program_can_and_keeps_the_ai_map() -> XrfResult {
  let sdk: PathBuf = sdk()?;

  assert_eq!(
    arguments(&sdk, LevelCompileStep::Geometry)?,
    Some((
      vec![
        String::from("-f"),
        String::from("mp_darkvalley"),
        String::from("-silent")
      ],
      true
    ))
  );
  assert_eq!(
    arguments(&sdk, LevelCompileStep::Ai)?,
    Some((
      vec![
        String::from("-f"),
        String::from("mp_darkvalley"),
        String::from("-keep_temp_files")
      ],
      false
    ))
  );

  Ok(())
}

#[test]
fn runs_an_sdk_program_for_exactly_the_steps_the_sdk_runs() -> XrfResult {
  let sdk: PathBuf = sdk()?;

  assert_eq!(arguments(&sdk, LevelCompileStep::Details)?, None);

  for step in LevelCompileStep::ALL {
    assert_eq!(LevelSdkProgram::of(&sdk, step)?.is_some(), step.is_sdk(), "{step:?}");
  }

  Ok(())
}
