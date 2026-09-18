use serde::Serialize;
use xrf_efd::{EfdPattern, EfdVariable};

/// One term of an evaluation function, as the viewer reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEfdPattern {
  /// Inputs the term reads, by their positions in the function's own variable list.
  pub variables: Vec<u32>,
  /// Weights the term claims, which is the product of its inputs' ranges and a number the file never stores.
  pub weights: Option<u64>,
}

impl ArchiveEfdPattern {
  /// Every term of a function, in the order it declares them.
  pub fn of_all(patterns: &[EfdPattern], variables: &[EfdVariable]) -> Vec<Self> {
    patterns
      .iter()
      .map(|pattern| Self {
        variables: pattern.variables.clone(),
        weights: pattern.get_complexity(variables),
      })
      .collect()
  }
}
