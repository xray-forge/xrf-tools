use serde::{Deserialize, Serialize};

use crate::efd::efd_variable::EfdVariable;

/// One term of an evaluation function: the inputs it reads, by their positions in the function's own variable list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EfdPattern {
  pub variables: Vec<u32>,
}

impl EfdPattern {
  /// Bytes a pattern occupies before its indices.
  pub const FIXED_SIZE: u64 = 4;

  /// Parameters this pattern claims, which is the product of its inputs' ranges.
  pub fn get_complexity(&self, variables: &[EfdVariable]) -> Option<u64> {
    self.variables.iter().try_fold(1u64, |complexity, index| {
      variables
        .get(*index as usize)
        .map(|variable| complexity.saturating_mul(u64::from(variable.range)))
    })
  }
}
