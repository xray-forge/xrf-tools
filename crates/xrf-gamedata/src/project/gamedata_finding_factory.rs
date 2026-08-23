use std::cmp::Ordering;
use std::path::Path;

use xrf_report::RuleId;

use crate::{Finding, GamedataVerificationRule};
use xrf_utils::to_portable_path_string;

/// Constructs shared report findings from gamedata verification inputs.
pub(crate) struct GamedataFindingFactory;

impl GamedataFindingFactory {
  pub(crate) fn for_asset<P, M>(rule: GamedataVerificationRule, asset_path: P, message: M) -> Finding
  where
    P: AsRef<Path>,
    M: Into<String>,
  {
    Self::create(rule, Some(to_portable_path_string(asset_path)), message)
  }

  pub(crate) fn without_asset<M>(rule: GamedataVerificationRule, message: M) -> Finding
  where
    M: Into<String>,
  {
    Self::create(rule, None, message)
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

  fn create<M>(rule: GamedataVerificationRule, subject: Option<String>, message: M) -> Finding
  where
    M: Into<String>,
  {
    Finding::new(
      RuleId::new(rule.to_string()).expect("Gamedata verification rules have non-empty stable identifiers"),
      subject,
      message,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataFindingFactory;
  use crate::{Finding, GamedataVerificationRule};

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
