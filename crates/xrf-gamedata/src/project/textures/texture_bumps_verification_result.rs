use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

/// Aggregated outcome of resolving every `.thm` bump declaration the way the renderer would.
#[derive(Default)]
pub(crate) struct GamedataTextureBumpsVerificationResult {
  pub(crate) findings: Vec<Finding>,
  /// Texture descriptors whose declaration makes the renderer bind a bump pair.
  pub(crate) checked_bumps_count: u32,
  /// Declared bumps whose own file is not the one the declaration names: the engine draws a dummy or the not-existing
  /// texture, so the surface is flat while paying for the bump shader.
  pub(crate) unresolved_bumps_count: u32,
  /// Declared bumps that resolved but whose `#` companion is not the file the declaration names. The engine binds
  /// `ed\ed_dummy_bump#`: parallax relief is lost and the DXT normal goes uncorrected, but the surface still reads as
  /// bumped, and vanilla ships one such pair. A pair missing both halves counts once, above. Reported always, gating
  /// only under strict.
  pub(crate) missing_companions_count: u32,
  /// Descriptors that ask for a bump the engine never binds: a texture type `LoadTHM` skips, or a used mode with no
  /// name. Authoring that does nothing, harmless at runtime; vanilla ships two. Reported always, gating only under strict.
  pub(crate) invalid_bump_declarations_count: u32,
  /// Whether the run was asked to judge what the engine tolerates silently, which is what promotes a missing companion
  /// and an unread declaration from counted to failing.
  pub(crate) is_strict: bool,
}

impl GamedataCheckResult for GamedataTextureBumpsVerificationResult {
  fn get_status(&self) -> GamedataVerificationStatus {
    // Default mode fails on what the engine reads and renders wrong: a bump it binds a substitute for. Strict also
    // judges what the engine tolerates silently: a companion it substitutes, and a declaration it never reads.
    GamedataVerificationStatus::from_is_valid(
      self.unresolved_bumps_count == 0
        && (!self.is_strict || (self.missing_companions_count == 0 && self.invalid_bump_declarations_count == 0)),
    )
  }

  fn get_failure_message(&self) -> String {
    let mut message: String = format!(
      "{}/{} declared bumps resolved",
      self.checked_bumps_count - self.unresolved_bumps_count,
      self.checked_bumps_count
    );

    // Only when there is something to say: the common case reads as it always has.
    if self.missing_companions_count > 0 {
      message.push_str(&format!(
        ", {} bumps without a companion",
        self.missing_companions_count
      ));
    }

    if self.invalid_bump_declarations_count > 0 {
      message.push_str(&format!(
        ", {} bump declarations the engine never reads",
        self.invalid_bump_declarations_count
      ));
    }

    message
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataTextureBumpsVerificationResult;
  use crate::{GamedataCheckResult, GamedataVerificationStatus};

  #[test]
  fn a_missing_bump_fails_in_every_mode() {
    // The engine binds a substitute for a texture it does read: the surface is flat while paying for the bump shader.
    let result: GamedataTextureBumpsVerificationResult = GamedataTextureBumpsVerificationResult {
      checked_bumps_count: 2,
      unresolved_bumps_count: 1,
      ..Default::default()
    };

    assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(result.get_failure_message(), "1/2 declared bumps resolved");

    let clean: GamedataTextureBumpsVerificationResult = GamedataTextureBumpsVerificationResult {
      checked_bumps_count: 1,
      ..Default::default()
    };

    assert_eq!(clean.get_status(), GamedataVerificationStatus::Passed);
    assert_eq!(clean.get_failure_message(), "1/1 declared bumps resolved");
  }

  #[test]
  fn an_unread_declaration_is_named_always_and_fails_only_under_strict() {
    // The engine never reads it, so nothing renders wrong; vanilla ships two, so a default run stays green.
    let lenient: GamedataTextureBumpsVerificationResult = GamedataTextureBumpsVerificationResult {
      checked_bumps_count: 1,
      invalid_bump_declarations_count: 1,
      ..Default::default()
    };

    assert_eq!(lenient.get_status(), GamedataVerificationStatus::Passed);
    assert_eq!(
      lenient.get_failure_message(),
      "1/1 declared bumps resolved, 1 bump declarations the engine never reads"
    );

    let strict: GamedataTextureBumpsVerificationResult = GamedataTextureBumpsVerificationResult {
      is_strict: true,
      ..lenient
    };

    assert_eq!(strict.get_status(), GamedataVerificationStatus::Failed);
  }

  #[test]
  fn a_missing_companion_is_named_always_and_fails_only_under_strict() {
    // The engine binds a flat dummy companion and keeps drawing a bumped surface; vanilla ships one such pair, so a
    // default run reports it without failing and a strict run judges it.
    let lenient: GamedataTextureBumpsVerificationResult = GamedataTextureBumpsVerificationResult {
      checked_bumps_count: 1,
      missing_companions_count: 1,
      ..Default::default()
    };

    assert_eq!(lenient.get_status(), GamedataVerificationStatus::Passed);
    assert_eq!(
      lenient.get_failure_message(),
      "1/1 declared bumps resolved, 1 bumps without a companion"
    );

    let strict: GamedataTextureBumpsVerificationResult = GamedataTextureBumpsVerificationResult {
      is_strict: true,
      ..lenient
    };

    assert_eq!(strict.get_status(), GamedataVerificationStatus::Failed);
  }
}
