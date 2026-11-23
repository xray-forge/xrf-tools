use std::time::Instant;

use xrf_error::XrfResult;

use crate::project::gamedata_check::GamedataCheck;
use crate::project::gamedata_finding_factory::GamedataFindingFactory;
use crate::project::gamedata_project::GamedataProject;
use crate::project::gamedata_project_verify_options::GamedataProjectVerifyOptions;
use crate::project::gamedata_verification_rule::GamedataVerificationRule;
use crate::project::info_portions::info_portion_place::InfoPortionPlace;
use crate::project::info_portions::info_portion_spelling::InfoPortionSpelling;
use crate::project::info_portions::info_portion_table::InfoPortionTable;
use crate::project::info_portions::info_portion_uses::InfoPortionUses;
use crate::project::info_portions::verify_info_portions_result::GamedataInfoPortionsVerificationResult;

/// The info portions check: every info portion something reads is given somewhere, every one something gives is read or
/// does something by being held, and every one declared to do something is given.
///
/// The engine accepts any spelling and never says a portion it reads is one nothing gives, so a misspelling between
/// the side giving a portion and the side reading it stalls a quest silently. What this check reports is therefore
/// always a warning.
pub(crate) struct InfoPortionsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for InfoPortionsVerifier<'a> {
  type Result = GamedataInfoPortionsVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataInfoPortionsVerificationResult> {
    self.options.job.check_cancelled()?;

    let started_at: Instant = Instant::now();
    let table: InfoPortionTable = InfoPortionTable::read(self.project, &self.options.job)?;
    let mut result: GamedataInfoPortionsVerificationResult = GamedataInfoPortionsVerificationResult {
      checked_info_portions_count: u32::try_from(table.get_uses().len()).unwrap_or(u32::MAX),
      unread_configs_count: table.get_unread_configs_count(),
      unread_scripts_count: table.get_unread_scripts_count(),
      has_scripts: table.get_scripts_count() > 0,
      ..GamedataInfoPortionsVerificationResult::default()
    };

    self.options.job.check_cancelled()?;

    // Scripts give portions too, so in a tree holding none, such as a mod relying on the game's scripts, nothing tells
    // which portions are never given.
    if result.has_scripts {
      let known: Vec<&str> = table.list_known_names();

      for (name, uses) in table.get_uses() {
        let Some(read) = uses.find_read_only() else {
          continue;
        };
        let mut message: String = Self::describe_read_only(name, read, uses);

        if let Some(closest) = InfoPortionSpelling::find_closest(name, known.iter().copied()) {
          message.push_str(&format!("; did you mean '{closest}'?"));
        }

        result.read_only_count += 1;
        result.findings.push(GamedataFindingFactory::warning_for_asset(
          GamedataVerificationRule::InfoPortionReadOnly,
          &read.path,
          message,
        ));
      }

      // State nothing acts on: a portion given and never read, whose declaration opens no dialog and takes nothing.
      for (name, uses) in table.get_uses() {
        let Some(given) = uses.find_unread() else {
          continue;
        };

        result.unread_count += 1;
        result.findings.push(GamedataFindingFactory::warning_for_asset(
          GamedataVerificationRule::InfoPortionUnread,
          &given.path,
          format!(
            "Info portion '{name}' is given by {}, but no condlist, dialog or script reads it and holding it does \
             nothing",
            given.context
          ),
        ));
      }

      // Behaviour that never runs: a declaration that opens dialogs or takes portions away, for a portion nobody gives.
      for (name, uses) in table.get_uses() {
        let Some(declared) = uses.find_unreachable() else {
          continue;
        };

        result.unreachable_count += 1;
        result.findings.push(GamedataFindingFactory::warning_for_asset(
          GamedataVerificationRule::InfoPortionUnreachable,
          &declared.path,
          format!(
            "Info portion '{name}' is declared to open dialogs or take other portions away, but nothing gives it, so \
             that never happens"
          ),
        ));
      }
    }

    result
      .findings
      .sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);
    result.duration = started_at.elapsed();

    Ok(result)
  }
}

impl InfoPortionsVerifier<'_> {
  /// What a portion read and never given is told: where it is first read, and the dialog file the engine never loads
  /// that would give it.
  fn describe_read_only(name: &str, read: &InfoPortionPlace, uses: &InfoPortionUses) -> String {
    let mut message: String = format!(
      "Info portion '{name}' is read by {}, but no condlist, dialog or spawned document gives it and no script names it",
      read.context
    );

    if let Some(unlisted) = &uses.given_unlisted {
      message.push_str(&format!(
        "; {} would, but system.ltx does not list its file",
        unlisted.context
      ));
    }

    message
  }
}
