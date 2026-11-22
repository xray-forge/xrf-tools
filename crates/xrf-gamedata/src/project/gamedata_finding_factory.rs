use std::cmp::Ordering;
use std::path::Path;

use xrf_report::{Finding, RuleId};
use xrf_utils::format_portable_path;

use crate::project::gamedata_verification_rule::GamedataVerificationRule;

/// Constructs shared report findings from gamedata verification inputs.
pub(crate) struct GamedataFindingFactory;

impl GamedataFindingFactory {
  /// A failure about one asset: what the engine cannot work with.
  pub(crate) fn for_asset<P, M>(rule: GamedataVerificationRule, asset_path: P, message: M) -> Finding
  where
    P: AsRef<Path>,
    M: Into<String>,
  {
    Finding::failure(Self::rule_id(rule), Some(format_portable_path(asset_path)), message)
  }

  /// A warning about one asset: what the engine tolerates, which fails the check only when the run is strict.
  pub(crate) fn warning_for_asset<P, M>(rule: GamedataVerificationRule, asset_path: P, message: M) -> Finding
  where
    P: AsRef<Path>,
    M: Into<String>,
  {
    Finding::warning(Self::rule_id(rule), Some(format_portable_path(asset_path)), message)
  }

  pub(crate) fn without_asset<M>(rule: GamedataVerificationRule, message: M) -> Finding
  where
    M: Into<String>,
  {
    Finding::failure(Self::rule_id(rule), None, message)
  }

  /// Orders findings by asset path, then message.
  pub(crate) fn cmp_by_asset_path_and_message(left: &Finding, right: &Finding) -> Ordering {
    left
      .subject()
      .cmp(&right.subject())
      .then_with(|| left.message().cmp(right.message()))
  }

  /// Orders findings by asset path, rule, then message.
  pub(crate) fn cmp_by_asset_path_rule_and_message(left: &Finding, right: &Finding) -> Ordering {
    left
      .subject()
      .cmp(&right.subject())
      .then_with(|| left.rule_id().cmp(right.rule_id()))
      .then_with(|| left.message().cmp(right.message()))
  }

  fn rule_id(rule: GamedataVerificationRule) -> RuleId {
    RuleId::new(rule.to_string()).expect("Gamedata verification rules have non-empty stable identifiers")
  }
}

#[cfg(test)]
mod tests {
  use xrf_report::Finding;

  use super::GamedataFindingFactory;
  use crate::project::gamedata_verification_rule::GamedataVerificationRule;

  #[test]
  fn sorts_findings_by_asset_path_then_message() {
    let mut findings: Vec<Finding> = vec![
      GamedataFindingFactory::for_asset(GamedataVerificationRule::ScriptsSyntax, "scripts/a.script", "Second"),
      GamedataFindingFactory::for_asset(GamedataVerificationRule::ScriptsSyntax, "scripts/z.script", "First"),
      GamedataFindingFactory::for_asset(GamedataVerificationRule::ScriptsSyntax, "scripts/a.script", "First"),
    ];

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    assert_eq!(
      findings.iter().map(Finding::message).collect::<Vec<_>>(),
      vec!["First", "Second", "First"]
    );
  }

  #[test]
  fn sorts_equal_paths_by_rule_before_message() {
    let scripts_finding: Finding =
      GamedataFindingFactory::for_asset(GamedataVerificationRule::ScriptsSyntax, "scripts/a.script", "First");
    let textures_finding: Finding =
      GamedataFindingFactory::for_asset(GamedataVerificationRule::TexturesRead, "scripts/a.script", "Second");
    let mut findings: Vec<Finding> = vec![textures_finding.clone(), scripts_finding.clone()];

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);

    assert_eq!(findings, vec![scripts_finding, textures_finding]);
  }
}
