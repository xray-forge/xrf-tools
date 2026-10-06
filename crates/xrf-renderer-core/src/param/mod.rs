//! A pass's parameters: one bind group declared as a struct, from which its layout, WGSL declarations, graph accesses
//! and bindings come, and the cache its bind groups are kept in.

mod bind_group_cache;
mod bind_group_key;
mod binding_key;
mod field;
mod pass_binding;
mod pass_binding_layout;
mod pass_parameters;
mod pass_resources;
mod shader_bindings;

#[cfg(test)]
mod tests;

pub use bind_group_cache::BindGroupCache;
pub use field::{
  StorageArray, StorageArrayMut, StorageField, StorageValue, StorageValueMut, UniformBinding, UniformField,
};
pub use pass_binding::PassBinding;
pub use pass_binding_layout::PassBindingLayout;
pub use pass_parameters::PassParameters;
pub use pass_resources::PassResources;
pub use shader_bindings::ShaderBindings;
