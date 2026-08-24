use std::io::Cursor;
use std::path::Path;
use std::time::{Duration, Instant};

use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};
use xrf_lua::verify_luajit_script;
use xrf_utils::read_as_string_from_w1251_encoded;
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::scripts::runtime_script::is_runtime_script;
use crate::project::scripts::verify_scripts_result::GamedataScriptsVerificationResult;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

impl GamedataProject {
  pub fn verify_scripts(&self, options: &GamedataProjectVerifyOptions) -> XrfResult<GamedataScriptsVerificationResult> {
    xrf_output::heading!(options.output, "Verify scripts:");

    let started_at: Instant = Instant::now();
    let script_paths: Vec<String> = self
      .entries_of_type(AssetType::Script)
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .filter(|path| is_runtime_script(path))
      .collect();

    let checked_scripts_count: u32 = u32::try_from(script_paths.len())
      .map_err(|_| XrfError::new_verify_error("Script count exceeds the supported result range"))?;

    let mut findings: Vec<Finding> = script_paths
      .par_iter()
      .filter_map(|relative_path| {
        xrf_output::verbose!(options.output, "Verify script: {relative_path}");

        let Some(path) = self
          .find(relative_path)
          .ok()
          .flatten()
          .map(|location| location.get_logical_path().to_string())
        else {
          xrf_output::info!(options.output, "Script path not found: {relative_path}");

          return Some(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::ScriptsPath,
            Path::new(relative_path),
            "Script path was not found in gamedata roots",
          ));
        };

        match self.verify_script(options, &path) {
          Ok(true) => None,
          Ok(false) => {
            xrf_output::info!(options.output, "Script is not valid: {}", path);

            Some(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::ScriptsSyntax,
              &path,
              "LuaJIT parser rejected the script",
            ))
          }
          Err(error) => {
            xrf_output::error!(options.output, "Script verification failed: {error}");

            Some(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::ScriptsRead,
              &path,
              error.to_string(),
            ))
          }
        }
      })
      .collect();

    let duration: Duration = started_at.elapsed();
    let invalid_scripts_count: u32 = u32::try_from(findings.len())
      .map_err(|_| XrfError::new_verify_error("Invalid script count exceeds the supported result range"))?;

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    if checked_scripts_count > 0 {
      xrf_output::info!(
        options.output,
        "Verified gamedata scripts in {}, {}/{} valid",
        xrf_utils::format_duration(duration),
        checked_scripts_count - invalid_scripts_count,
        checked_scripts_count
      );
    } else {
      xrf_output::info!(
        options.output,
        "Check gamedata scripts in {}, no scripts found",
        xrf_utils::format_duration(duration),
      );
    }

    Ok(GamedataScriptsVerificationResult {
      duration,
      checked_scripts_count,
      findings,
      invalid_scripts_count,
    })
  }

  /// Parses one script, addressed by its logical path.
  ///
  /// Read through the VFS, so an archived script is parsed rather than reported missing.
  pub fn verify_script(&self, _options: &GamedataProjectVerifyOptions, logical_path: &str) -> XrfResult<bool> {
    let bytes: Vec<u8> = self.read_bytes(logical_path)?;
    let code: String = read_as_string_from_w1251_encoded(&mut Cursor::new(bytes))?;

    verify_luajit_script(&code, Path::new(logical_path))?;

    Ok(true)
  }
}
