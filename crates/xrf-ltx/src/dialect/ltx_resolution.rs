use std::collections::BTreeMap;

use crate::Ltx;

/// What resolving one root answers with.
///
/// Carries more than the resolved document because a dialect that patches configs has to explain itself: in a modded
/// install the only way to account for a value is to name the file that won it.
#[derive(Debug, Default)]
pub struct LtxResolution {
  pub ltx: Ltx,
  /// Where each resolved field came from, keyed by section and key.
  ///
  /// Empty under standard LTX, where every value comes from the one file that declared it and nothing was in
  /// contention.
  pub provenance: BTreeMap<(String, String), LtxFieldOrigin>,
  /// What the dialect found worth saying that is not a failure.
  pub diagnostics: Vec<LtxResolutionDiagnostic>,
}

impl LtxResolution {
  /// A resolution with nothing to explain, which is what standard LTX answers.
  pub fn new_plain(ltx: Ltx) -> Self {
    Self {
      diagnostics: Vec::new(),
      ltx,
      provenance: BTreeMap::new(),
    }
  }

  /// Where one field came from, when the dialect tracks it.
  pub fn get_origin(&self, section: &str, key: &str) -> Option<&LtxFieldOrigin> {
    self.provenance.get(&(String::from(section), String::from(key)))
  }
}

/// Which file supplied one resolved field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LtxFieldOrigin {
  /// Lowercased base name of the winning file.
  pub file: String,
  /// Load rank of the winning statement. Negative means a patch file outranking the base tree.
  pub depth: i32,
  /// How the value was written, spelled as its prefix: empty for a plain assignment, `>` for a list append.
  pub operation: String,
}

/// Something a dialect wants said about a config tree, short of refusing it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LtxResolutionDiagnostic {
  pub section: String,
  pub file: Option<String>,
  pub message: String,
  /// What the engine does with the same input, where that differs from reporting it.
  ///
  /// The reason these are worth emitting at all: a modder cannot see what the game silently drops, so a warning that
  /// says "the game loads this and says nothing" is more use than the same warning without it.
  pub engine_behaviour: Option<String>,
}
