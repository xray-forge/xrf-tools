use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level_compile_step::LevelCompileStep;

/// The SDK 0.7 compiler that runs a step, under the SDK's `editors\bin`, the prefix of the log it writes under its
/// working directory's `logs`, and the switches it runs with after `-f <level>`.
///
/// Each compiler ends a run on a "Congratulation!" message box and waits on it. xrLC skips it under `-silent`; xrAI has no such switch, so its box ends the run (`LevelSdkDialog`). xrAI deletes `build.aimap` once
/// read unless given `-keep_temp_files` (`xrAI/compiler_load.cpp:318`), which keeps the level a step can run on again;
/// xrLC takes the same switch to write debug files, so it runs without.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LevelSdkProgram {
  pub(crate) path: PathBuf,
  pub(crate) log_prefix: &'static str,
  pub(crate) switches: &'static [&'static str],
}

impl LevelSdkProgram {
  /// The switch that skips the closing message box.
  const SILENT: &'static str = "-silent";

  /// The SDK's program for `step` under `sdk`, the SDK's `editors` directory, where the SDK runs it
  /// ([`LevelCompileStep::is_sdk`]); none for a step of ours.
  ///
  /// # Errors
  ///
  /// Returns an error naming the program when the SDK holds no such file.
  pub(crate) fn of(sdk: &Path, step: LevelCompileStep) -> XrfResult<Option<Self>> {
    let (directory, name, log_prefix, switches): (&[&str], &str, &str, &'static [&'static str]) = match step {
      LevelCompileStep::Geometry => (&["bin", "compilers"], "xrLC.exe", "xrlc_", &[Self::SILENT]),
      LevelCompileStep::Ai => (
        &["bin", "designer", "release"],
        "xrAI.exe",
        "xrai_",
        &["-keep_temp_files"],
      ),
      LevelCompileStep::Details => return Ok(None),
    };
    let path: PathBuf = directory
      .iter()
      .fold(sdk.to_path_buf(), |path, part| path.join(part))
      .join(name);

    if !path.is_file() {
      return Err(XrfError::new_not_found_error(format!(
        "The SDK's {name} is not at {}; name the SDK's editors directory",
        format_path(&path)
      )));
    }

    Ok(Some(Self {
      path,
      log_prefix,
      switches,
    }))
  }

  /// The arguments the program takes for the level named `level`: `-f` and the level, which all three compilers take,
  /// then its switches.
  pub(crate) fn get_arguments(&self, level: &str) -> Vec<String> {
    [String::from("-f"), level.to_owned()]
      .into_iter()
      .chain(self.switches.iter().map(|switch| String::from(*switch)))
      .collect()
  }

  /// Whether the program skips its closing message box, exiting on its own.
  pub(crate) fn is_silent(&self) -> bool {
    self.switches.contains(&Self::SILENT)
  }
}
