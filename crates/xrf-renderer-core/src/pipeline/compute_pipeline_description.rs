use crate::pipeline::pipeline_constants::PipelineConstants;
use crate::pipeline::shader_permutation::ShaderPermutation;

/// Everything a compute pipeline is made from, and the key the pipeline cache keeps it by.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComputePipelineDescription {
  pub label: &'static str,
  pub module: &'static str,
  pub entry_point: &'static str,
  pub constants: PipelineConstants,
  /// Its bind group layouts by index, a gap where a slot binds nothing, compared by identity.
  pub bind_group_layouts: Vec<Option<wgpu::BindGroupLayout>>,
}

impl ComputePipelineDescription {
  pub fn new(label: &'static str, module: &'static str, entry_point: &'static str) -> Self {
    Self {
      label,
      module,
      entry_point,
      constants: PipelineConstants::default(),
      bind_group_layouts: Vec::new(),
    }
  }

  pub fn with_permutation(mut self, permutation: &impl ShaderPermutation) -> Self {
    self.constants = PipelineConstants::of(permutation);
    self
  }

  /// Its bind group layouts by index, a gap where a slot binds nothing.
  pub fn with_bind_group_layouts(mut self, layouts: &[Option<&wgpu::BindGroupLayout>]) -> Self {
    self.bind_group_layouts = layouts.iter().map(|layout| layout.cloned()).collect();
    self
  }
}
