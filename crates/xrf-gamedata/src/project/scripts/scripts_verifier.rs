use std::io::Cursor;
use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;
use xrf_job::parallel::prelude::*;
use xrf_lua::XRayLuaScript;
use xrf_report::Finding;
use xrf_utils::read_as_string_from_w1251_encoded;
use xrf_vfs::{XrayAsset, XrayAssetType};

use crate::project::gamedata_asset_sweep::GamedataAssetSweep;
use crate::project::gamedata_asset_sweep_outcome::GamedataAssetSweepOutcome;
use crate::project::gamedata_check::GamedataCheck;
use crate::project::gamedata_finding_factory::GamedataFindingFactory;
use crate::project::gamedata_project::GamedataProject;
use crate::project::gamedata_project_verify_options::GamedataProjectVerifyOptions;
use crate::project::gamedata_verification_rule::GamedataVerificationRule;
use crate::project::scripts::gamedata_parsed_script::GamedataParsedScript;
use crate::project::scripts::verify_scripts_result::GamedataScriptsVerificationResult;

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

    let started_at: Instant = Instant::now();
    let scripts: Vec<XrayAsset> = Self::list_runtime_scripts(self.project);

    let outcome: GamedataAssetSweepOutcome =
      GamedataAssetSweep::new(self.options).run(&scripts, |script| self.verify_script(script).into_iter().collect())?;

    self.options.job.check_cancelled()?;

    let duration: Duration = started_at.elapsed();

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

  /// Every script asset holding executable Lua the game runtime loads, in logical-path order.
  pub(crate) fn list_runtime_scripts(project: &GamedataProject) -> Vec<XrayAsset> {
    project
      .entries_of_type(XrayAssetType::Script)
      .into_iter()
      .filter(|script| Self::is_runtime_script(script.get_logical_path().as_str()))
      .collect()
  }

  /// Every runtime script read and parsed, in parallel, for a check reading what scripts define or name; one not
  /// reached before the job is cancelled is left out.
  pub(crate) fn parse_runtime_scripts(project: &GamedataProject, job: &JobHandle) -> Vec<GamedataParsedScript> {
    Self::list_runtime_scripts(project)
      .into_par_iter()
      .filter_map(|script| {
        if job.is_cancelled() {
          return None;
        }

        Some(GamedataParsedScript {
          path: script.get_logical_path().to_string(),
          parse: Self::read_parsed(project, &script),
        })
      })
      .collect()
  }

  /// What parsing a script's code came to, the code read as the game reads it: through the VFS, so an archived script
  /// is read too, and decoded from windows-1251.
  ///
  /// The project's snapshot retains the parse, so every check reading the script meets the same one, and a script
  /// LuaJIT refuses is refused once. A script that does not read is answered as an error and not retained.
  fn read_parsed(project: &GamedataProject, script: &XrayAsset) -> XrfResult<Arc<XrfResult<XRayLuaScript>>> {
    let path: &str = script.get_logical_path().as_str();

    project.read_resolved_parsed(script, |bytes| {
      let code: String = read_as_string_from_w1251_encoded(&mut Cursor::new(bytes))?;

      Ok(XRayLuaScript::parse(path, &code))
    })
  }

  /// Whether a script asset holds executable Lua the game runtime loads.
  fn is_runtime_script(path: &str) -> bool {
    !path.eq_ignore_ascii_case(Self::LUA_HELP_SCRIPT_PATH)
  }

  /// The finding one script raises: it does not read, or LuaJIT rejects it.
  fn verify_script(&self, script: &XrayAsset) -> Option<Finding> {
    let path: &str = script.get_logical_path().as_str();
    let parse: Arc<XrfResult<XRayLuaScript>> = match Self::read_parsed(self.project, script) {
      Ok(parse) => parse,
      Err(error) => {
        return Some(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::ScriptsRead,
          path,
          error.to_string(),
        ));
      }
    };
    let error: &XrfError = parse.as_ref().as_ref().err()?;

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
