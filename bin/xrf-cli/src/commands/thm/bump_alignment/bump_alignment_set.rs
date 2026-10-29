use serde::Serialize;
use xrf_texture::BumpAlignment;

/// One descriptor's diffuse and the bump it declares, and whether the two belong together.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpAlignmentSet {
  /// The descriptor declaring the bump.
  pub descriptor: String,
  /// The diffuse beside it, where a root holds one.
  pub diffuse: Option<String>,
  /// The bump as the descriptor names it.
  pub bump: String,
  /// The file the bump resolved to, where a root holds it.
  pub bump_file: Option<String>,
  /// Whether the bump is another texture's rather than the diffuse's own `<texture>_bump`: a descriptor borrowing a
  /// similar surface's relief, which follows the diffuse only as far as the two pictures agree.
  pub is_borrowed: bool,
  #[serde(flatten)]
  pub outcome: BumpAlignmentOutcome,
}

/// What comparing one set came to.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum BumpAlignmentOutcome {
  Measured {
    alignment: BumpAlignment,
  },
  /// No root holds the diffuse the descriptor describes, as for a bump's own descriptor kept beside a removed texture.
  MissingDiffuse,
  /// No root holds the bump: the renderer binds its stand-in, which `gamedata verify` reports.
  MissingBump,
  Unread {
    reason: String,
  },
}

impl BumpAlignmentSet {
  /// Its measure, where it was measured.
  pub fn get_alignment(&self) -> Option<&BumpAlignment> {
    match &self.outcome {
      BumpAlignmentOutcome::Measured { alignment } => Some(alignment),
      _ => None,
    }
  }
}
