/// A vertex buffer's layout, owned so a pipeline description holding it can be a cache key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertexLayout {
  pub array_stride: u64,
  pub step_mode: wgpu::VertexStepMode,
  pub attributes: Vec<wgpu::VertexAttribute>,
}

impl VertexLayout {
  pub fn new(array_stride: u64, step_mode: wgpu::VertexStepMode, attributes: &[wgpu::VertexAttribute]) -> Self {
    Self {
      array_stride,
      step_mode,
      attributes: attributes.to_vec(),
    }
  }

  pub(crate) fn to_wgpu(&self) -> wgpu::VertexBufferLayout<'_> {
    wgpu::VertexBufferLayout {
      array_stride: self.array_stride,
      step_mode: self.step_mode,
      attributes: &self.attributes,
    }
  }
}
