use crate::param::pass_binding::PassBinding;

/// What one binding of a cached bind group binds, by the resource's identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BindingKey {
  /// A buffer, with the offset and size of the range bound where it binds one.
  Buffer(wgpu::Buffer, Option<(u64, u64)>),
  TextureView(wgpu::TextureView),
  TextureViewArray(Vec<wgpu::TextureView>),
  Sampler(wgpu::Sampler),
  /// An acceleration structure, by the key it was made with.
  AccelerationStructure(u64),
}

impl BindingKey {
  /// Whether it binds what `binding` names: the same resources, and a range's same offset and size.
  pub(crate) fn is_of(&self, binding: &PassBinding<'_>) -> bool {
    match (self, binding) {
      (Self::Buffer(buffer, None), PassBinding::Buffer(other)) => buffer == *other,
      (
        Self::Buffer(buffer, Some(range)),
        PassBinding::BufferRange {
          buffer: other,
          offset,
          size,
        },
      ) => buffer == *other && *range == (*offset, *size),
      (Self::TextureView(view), PassBinding::TextureView(other)) => view == *other,
      (Self::TextureViewArray(views), PassBinding::TextureViewArray(others)) => {
        views.len() == others.len() && views.iter().zip(others).all(|(view, other)| view == *other)
      }
      (Self::Sampler(sampler), PassBinding::Sampler(other)) => sampler == *other,
      (Self::AccelerationStructure(key), PassBinding::AccelerationStructure(structure)) => *key == structure.key,
      _ => false,
    }
  }
}

impl From<&PassBinding<'_>> for BindingKey {
  fn from(binding: &PassBinding<'_>) -> Self {
    match binding {
      PassBinding::Buffer(buffer) => Self::Buffer((*buffer).clone(), None),
      PassBinding::BufferRange { buffer, offset, size } => Self::Buffer((*buffer).clone(), Some((*offset, *size))),
      PassBinding::TextureView(view) => Self::TextureView((*view).clone()),
      PassBinding::TextureViewArray(views) => {
        Self::TextureViewArray(views.iter().map(|view| (*view).clone()).collect())
      }
      PassBinding::Sampler(sampler) => Self::Sampler((*sampler).clone()),
      PassBinding::AccelerationStructure(structure) => Self::AccelerationStructure(structure.key),
    }
  }
}
