use std::io::Cursor;
use std::path::Path;
use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_lua::verify_luajit_script;
use xrf_utils::read_as_string_from_w1251_encoded;
use xrf_vfs::XrayAssetType;

use crate::project::gamedata_asset_sweep::GamedataAssetSweep;
use crate::project::gamedata_asset_sweep_outcome::GamedataAssetSweepOutcome;
use crate::project::gamedata_check::GamedataCheck;
use crate::project::scripts::verify_scripts_result::GamedataScriptsVerificationResult;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// The scripts check: every runtime script the game holds parses as LuaJIT.
pub(crate) struct ScriptsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for ScriptsVerifier<'a> {
  type Result = GamedataScriptsVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataScriptsVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify scripts:");

    let started_at: Instant = Instant::now();
    let script_paths: Vec<String> = self
      .project
      .entries_of_type(XrayAssetType::Script)
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .filter(|path| Self::is_runtime_script(path))
      .collect();

    let outcome: GamedataAssetSweepOutcome = GamedataAssetSweep::new(self.options)
      .run(&script_paths, |options, path| {
        self.verify_script(options, path).into_iter().collect()
      })?;

    self.options.job.check_cancelled()?;

    let duration: Duration = started_at.elapsed();

    if outcome.checked_count > 0 {
      xrf_output::info!(
        self.options.output,
        "Verified gamedata scripts in {}, {}/{} valid",
        xrf_utils::format_duration(duration),
        outcome.checked_count - outcome.invalid_count,
        outcome.checked_count
      );
    } else {
      xrf_output::info!(
        self.options.output,
        "Check gamedata scripts in {}, no scripts found",
        xrf_utils::format_duration(duration),
      );
    }

    Ok(GamedataScriptsVerificationResult {
      duration,
      checked_scripts_count: outcome.checked_count,
      findings: outcome.findings,
      invalid_scripts_count: outcome.invalid_count,
    })
  }
}

impl ScriptsVerifier<'_> {
  /// The engine API declaration generated for editor tooling. Its `global`, `namespace` and `class` declarations are not
  /// LuaJIT syntax, so it is not a runtime script and is not parsed as one.
  const LUA_HELP_SCRIPT_PATH: &'static str = r"scripts\lua_help.script";

  /// Whether a script asset holds executable Lua the game runtime loads.
  fn is_runtime_script(path: &str) -> bool {
    !path.eq_ignore_ascii_case(Self::LUA_HELP_SCRIPT_PATH)
  }

  /// The finding one script raises: it does not read, or LuaJIT rejects it. Read through the VFS, so an archived script
  /// is parsed rather than reported missing.
  fn verify_script(&self, options: &GamedataProjectVerifyOptions, path: &str) -> Option<Finding> {
    xrf_output::verbose!(options.output, "Verify script: {path}");

    let code: String = match self
      .project
      .read_bytes(path)
      .and_then(|bytes| Ok(read_as_string_from_w1251_encoded(&mut Cursor::new(bytes))?))
    {
      Ok(code) => code,
      Err(error) => {
        xrf_output::error!(options.output, "Script was not read: {path} - {error}");

        return Some(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::ScriptsRead,
          path,
          error.to_string(),
        ));
      }
    };

    let error = verify_luajit_script(&code, Path::new(path)).err()?;

    xrf_output::info!(options.output, "Script is not valid: {path}");

    Some(GamedataFindingFactory::for_asset(
      GamedataVerificationRule::ScriptsSyntax,
      path,
      error.to_string(),
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::ScriptsVerifier;

  #[test]
  fn excludes_lua_help_from_runtime_script_verification() {
    assert!(!ScriptsVerifier::is_runtime_script(r"scripts\lua_help.script"));
    assert!(!ScriptsVerifier::is_runtime_script(r"SCRIPTS\LUA_HELP.SCRIPT"));
    assert!(ScriptsVerifier::is_runtime_script(r"scripts\bind_stalker.script"));
  }
}
