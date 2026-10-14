use crate::pipeline::pipeline_constants::PipelineConstants;
use crate::pipeline::shader_permutation::ShaderPermutation;
use crate::pipeline::vertex_layout::VertexLayout;

/// Everything a render pipeline is made from, and the key the pipeline cache keeps it by.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderPipelineDescription {
  pub label: &'static str,
  pub module: &'static str,
  pub vertex_entry: &'static str,
  /// `None` for a depth-only pipeline.
  pub fragment_entry: Option<&'static str>,
  pub constants: PipelineConstants,
  /// Its bind group layouts by index, a gap where a slot binds nothing, compared by identity.
  pub bind_group_layouts: Vec<Option<wgpu::BindGroupLayout>>,
  pub vertex_layouts: Vec<VertexLayout>,
  pub primitive: wgpu::PrimitiveState,
  pub depth_stencil: Option<wgpu::DepthStencilState>,
  pub multisample: wgpu::MultisampleState,
  pub targets: Vec<Option<wgpu::ColorTargetState>>,
}

impl RenderPipelineDescription {
  /// A pipeline of one vertex entry point and no targets yet.
  pub fn new(label: &'static str, module: &'static str, vertex_entry: &'static str) -> Self {
    Self {
      label,
      module,
      vertex_entry,
      fragment_entry: None,
      constants: PipelineConstants::default(),
      bind_group_layouts: Vec::new(),
      vertex_layouts: Vec::new(),
      primitive: wgpu::PrimitiveState::default(),
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      targets: Vec::new(),
    }
  }

  pub fn with_fragment(mut self, entry: &'static str) -> Self {
    self.fragment_entry = Some(entry);
    self
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

  pub fn with_vertex_layout(mut self, layout: VertexLayout) -> Self {
    self.vertex_layouts.push(layout);
    self
  }

  pub fn with_primitive(mut self, primitive: wgpu::PrimitiveState) -> Self {
    self.primitive = primitive;
    self
  }

  pub fn with_depth_stencil(mut self, depth_stencil: wgpu::DepthStencilState) -> Self {
    self.depth_stencil = Some(depth_stencil);
    self
  }

  /// Its color targets in attachment order, a gap where a slot draws nothing.
  pub fn with_targets(mut self, targets: Vec<Option<wgpu::ColorTargetState>>) -> Self {
    self.targets = targets;
    self
  }

  pub fn with_target(mut self, target: wgpu::ColorTargetState) -> Self {
    self.targets.push(Some(target));
    self
  }
}
