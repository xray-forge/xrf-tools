//! NGX's C API on Vulkan and D3D12, as the public types use it.

#![expect(
  unsafe_code,
  reason = "drives the NGX SDK through its C bindings and wgpu-hal's raw handles"
)]

pub(crate) mod backend;
pub(crate) mod bindings;
pub(crate) mod feature;
pub(crate) mod ngx_application;
pub(crate) mod ngx_result;
pub(crate) mod sdk;
