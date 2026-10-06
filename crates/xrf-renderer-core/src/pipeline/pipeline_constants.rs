use crate::pipeline::shader_permutation::ShaderPermutation;

/// A permutation's override constants as a pipeline description holds them: by name, each value's bits, so the
/// description can be a cache key.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PipelineConstants {
  values: Vec<(&'static str, u64)>,
}

impl PipelineConstants {
  pub fn of(permutation: &impl ShaderPermutation) -> Self {
    Self {
      values: permutation
        .list_constants()
        .into_iter()
        .map(|(name, value)| (name, value.to_bits()))
        .collect(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.values.is_empty()
  }

  pub(crate) fn to_wgpu(&self) -> Vec<(&'static str, f64)> {
    self
      .values
      .iter()
      .map(|(name, bits)| (*name, f64::from_bits(*bits)))
      .collect()
  }
}
