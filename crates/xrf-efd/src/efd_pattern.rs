use serde::{Deserialize, Serialize};

use crate::efd_variable::EfdVariable;

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
  ///
  /// `None` when the pattern names an input past `variables`.
  pub fn get_complexity(&self, variables: &[EfdVariable]) -> Option<u64> {
    self.variables.iter().try_fold(1u64, |complexity, index| {
      variables
        .get(*index as usize)
        .map(|variable| complexity.saturating_mul(u64::from(variable.range)))
    })
  }

  /// Which of this pattern's own weights one bucketed value per function input selects.
  ///
  /// The values read as one mixed-radix number over the pattern's inputs, its first input the most significant, as
  /// `dwfGetPatternIndex` (`xrGame/ef_pattern.h`) builds it before adding the pattern's offset in the weight block.
  /// `None` when the pattern names an input past `variables` or `values`, or the index outgrows a `u64`.
  pub fn get_index(&self, values: &[u32], variables: &[EfdVariable]) -> Option<u64> {
    self.variables.iter().try_fold(0u64, |index, input| {
      let input: usize = *input as usize;

      index
        .checked_mul(u64::from(variables.get(input)?.range))?
        .checked_add(u64::from(*values.get(input)?))
    })
  }
}
