use std::hash::{DefaultHasher, Hash, Hasher};

use crate::param::binding_key::BindingKey;
use crate::param::pass_binding::PassBinding;

/// What a cached bind group binds: its layout and, binding by binding, the resources by identity. A group is found by
/// its hash and matched by `is_of`, both read from a frame's bindings as they are, so a lookup that finds it takes
/// nothing from them; the key itself is made only for a group made anew.
#[derive(Debug)]
pub(crate) struct BindGroupKey {
  pub layout: wgpu::BindGroupLayout,
  pub bindings: Vec<BindingKey>,
}

impl BindGroupKey {
  /// The key of a group binding `bindings` by `layout`, holding each resource.
  pub(crate) fn new(layout: &wgpu::BindGroupLayout, bindings: &[PassBinding<'_>]) -> Self {
    Self {
      layout: layout.clone(),
      bindings: bindings.iter().map(BindingKey::from).collect(),
    }
  }

  /// The hash a group binding `bindings` by `layout` is cached under: each resource's identity, and a range's offset
  /// and size.
  pub(crate) fn hash_of(layout: &wgpu::BindGroupLayout, bindings: &[PassBinding<'_>]) -> u64 {
    let mut hasher: DefaultHasher = DefaultHasher::new();

    layout.hash(&mut hasher);

    for binding in bindings {
      match binding {
        PassBinding::Buffer(buffer) => (0u8, *buffer).hash(&mut hasher),
        PassBinding::BufferRange { buffer, offset, size } => (1u8, *buffer, offset, size).hash(&mut hasher),
        PassBinding::TextureView(view) => (2u8, *view).hash(&mut hasher),
        PassBinding::TextureViewArray(views) => (3u8, views).hash(&mut hasher),
        PassBinding::Sampler(sampler) => (4u8, *sampler).hash(&mut hasher),
        PassBinding::AccelerationStructure(structure) => (5u8, structure.key).hash(&mut hasher),
      }
    }

    hasher.finish()
  }

  /// Whether it is the key of a group binding `bindings` by `layout`.
  pub(crate) fn is_of(&self, layout: &wgpu::BindGroupLayout, bindings: &[PassBinding<'_>]) -> bool {
    self.layout == *layout
      && self.bindings.len() == bindings.len()
      && self
        .bindings
        .iter()
        .zip(bindings)
        .all(|(key, binding)| key.is_of(binding))
  }
}
