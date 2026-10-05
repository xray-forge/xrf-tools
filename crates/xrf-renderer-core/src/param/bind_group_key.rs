use crate::param::binding_key::BindingKey;

/// What a cached bind group is found by: its layout and, binding by binding, what it binds.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct BindGroupKey {
  pub layout: wgpu::BindGroupLayout,
  pub bindings: Vec<BindingKey>,
}
