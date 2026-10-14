use crate::pipeline::shader_permutation::ShaderPermutation;

/// A permutation's override constants as a pipeline description holds them: by name, each value's bits, so the
/// description can be a cache key.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PipelineConstants {
  values: Vec<(&'static str, u64)>,
}

impl PipelineConstants {
  /// A permutation's override constants, as its derive lists them.
  pub fn of(permutation: &impl ShaderPermutation) -> Self {
    Self {
      values: permutation
        .list_constants()
        .into_iter()
        .map(|(name, value)| (name, value.to_bits()))
        .collect(),
    }
  }

  pub(crate) fn to_wgpu(&self) -> Vec<(&'static str, f64)> {
    self
      .values
      .iter()
      .map(|(name, bits)| (*name, f64::from_bits(*bits)))
      .collect()
  }
}
