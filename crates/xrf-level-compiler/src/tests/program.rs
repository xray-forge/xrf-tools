//! The command lines the SDK's compilers are started with.

use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::LevelCompileStep;
use crate::sdk::level_sdk_program::LevelSdkProgram;

/// An SDK `editors` directory whose compilers are empty files.
fn sdk() -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "sdk"))?;

  for (directory, program) in [
    ("bin/compilers", "xrLC.exe"),
    ("bin/compilers", "xrDO_Light.exe"),
    ("bin/designer/release", "xrAI.exe"),
  ] {
    std::fs::create_dir_all(root.join(directory))?;
    std::fs::write(root.join(directory).join(program), b"")?;
  }

  Ok(root)
}

fn arguments(sdk: &Path, step: LevelCompileStep) -> XrfResult<(Vec<String>, bool)> {
  let program: LevelSdkProgram = LevelSdkProgram::of(sdk, step)?;

  Ok((program.get_arguments("mp_darkvalley"), program.is_silent()))
}

#[test]
fn skips_the_closing_box_where_the_program_can_and_keeps_the_ai_map() -> XrfResult {
  let sdk: PathBuf = sdk()?;

  assert_eq!(
    arguments(&sdk, LevelCompileStep::Geometry)?,
    (
      vec![
        String::from("-f"),
        String::from("mp_darkvalley"),
        String::from("-silent")
      ],
      true
    )
  );
  assert_eq!(
    arguments(&sdk, LevelCompileStep::Details)?,
    (
      vec![
        String::from("-f"),
        String::from("mp_darkvalley"),
        String::from("-silent")
      ],
      true
    )
  );
  assert_eq!(
    arguments(&sdk, LevelCompileStep::Ai)?,
    (
      vec![
        String::from("-f"),
        String::from("mp_darkvalley"),
        String::from("-keep_temp_files")
      ],
      false
    )
  );

  Ok(())
}
