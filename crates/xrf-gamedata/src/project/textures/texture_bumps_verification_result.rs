use xrf_report::Finding;

use crate::project::gamedata_check_result::GamedataCheckResult;
use crate::project::gamedata_verification_status::GamedataVerificationStatus;

/// Aggregated outcome of resolving every `.thm` bump declaration the way the renderer would.
#[derive(Default)]
pub(crate) struct GamedataTextureBumpsVerificationResult {
  /// A failure for a bump the engine binds a substitute for or a descriptor that does not read, and a warning for
  /// what it tolerates: a missing companion, a declaration it never reads, bytes past a chunk's fields.
  pub(crate) findings: Vec<Finding>,
  /// Texture descriptors whose declaration makes the renderer bind a bump pair.
  pub(crate) checked_bumps_count: u32,
  /// Declared bumps whose own file is not the one the declaration names: the engine draws a dummy or the not-existing
  /// texture, so the surface is flat while paying for the bump shader.
  pub(crate) unresolved_bumps_count: u32,
  /// Declared bumps that resolved but whose `#` companion is not the file the declaration names. The engine binds
  /// `ed\ed_dummy_bump#`: parallax relief is lost and the DXT normal goes uncorrected, but the surface still reads as
  /// bumped, and vanilla ships one such pair. A pair missing both halves counts once, above.
  pub(crate) missing_companions_count: u32,
  /// Descriptors that ask for a bump the engine never binds: a texture type `LoadTHM` skips, or a used mode with no
  /// name. Authoring that does nothing, harmless at runtime; vanilla ships two.
  pub(crate) invalid_bump_declarations_count: u32,
  /// Descriptors holding bytes past a chunk's fields, which no engine reads and some texture packs write. Harmless at
  /// runtime.
  pub(crate) trailing_descriptors_count: u32,
}

impl GamedataCheckResult for GamedataTextureBumpsVerificationResult {
  fn get_status(&self) -> GamedataVerificationStatus {
    // What the engine reads and renders wrong fails; what it tolerates is a warning, which only a strict run fails on.
    GamedataVerificationStatus::from_findings(&self.findings)
  }

  fn get_summary(&self) -> String {
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

    if self.trailing_descriptors_count > 0 {
      message.push_str(&format!(
        ", {} descriptors with bytes past their fields",
        self.trailing_descriptors_count
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
  use xrf_report::Finding;

  use super::GamedataTextureBumpsVerificationResult;
  use crate::project::gamedata_check_result::GamedataCheckResult;
  use crate::project::gamedata_finding_factory::GamedataFindingFactory;
  use crate::project::gamedata_verification_rule::GamedataVerificationRule;
  use crate::project::gamedata_verification_status::GamedataVerificationStatus;

  fn finding(rule: GamedataVerificationRule) -> Finding {
    GamedataFindingFactory::for_asset(rule, "textures/a.thm", "message")
  }

  fn warning(rule: GamedataVerificationRule) -> Finding {
    GamedataFindingFactory::warning_for_asset(rule, "textures/a.thm", "message")
  }

  #[test]
  fn a_missing_bump_fails() {
    // The engine binds a substitute for a texture it does read: the surface is flat while paying for the bump shader.
    let result: GamedataTextureBumpsVerificationResult = GamedataTextureBumpsVerificationResult {
      checked_bumps_count: 2,
      unresolved_bumps_count: 1,
      findings: vec![finding(GamedataVerificationRule::TexturesBump)],
      ..Default::default()
    };

    assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(result.get_summary(), "1/2 declared bumps resolved");

    let clean: GamedataTextureBumpsVerificationResult = GamedataTextureBumpsVerificationResult {
      checked_bumps_count: 1,
      ..Default::default()
    };

    assert_eq!(clean.get_status(), GamedataVerificationStatus::Passed);
    assert_eq!(clean.get_summary(), "1/1 declared bumps resolved");
  }

  /// The engine works around each of these, and vanilla ships some, so a default run passes with them as warnings.
  #[test]
  fn passes_with_what_the_engine_tolerates_and_names_it() {
    let result: GamedataTextureBumpsVerificationResult = GamedataTextureBumpsVerificationResult {
      checked_bumps_count: 1,
      missing_companions_count: 1,
      invalid_bump_declarations_count: 1,
      trailing_descriptors_count: 2,
      findings: vec![
        warning(GamedataVerificationRule::TexturesBumpCompanion),
        warning(GamedataVerificationRule::TexturesBumpDeclaration),
        warning(GamedataVerificationRule::TexturesChunkResidue),
      ],
      ..Default::default()
    };

    assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
    assert_eq!(
      result.get_summary(),
      "1/1 declared bumps resolved, 1 bumps without a companion, 1 bump declarations the engine never reads, 2 \
       descriptors with bytes past their fields"
    );
  }
}
