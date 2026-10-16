//! NGX's C API on Vulkan and D3D12, as the public types use it.

pub(crate) mod bindings;
#[cfg(windows)]
pub(crate) mod d3d12_resource;
pub(crate) mod ngx_api;
pub(crate) mod ngx_application;
pub(crate) mod ngx_commands;
pub(crate) mod ngx_feature;
pub(crate) mod ngx_lock;
pub(crate) mod ngx_ray_reconstruction;
pub(crate) mod ngx_result;
pub(crate) mod ngx_retired_feature;
pub(crate) mod ngx_sdk;
pub(crate) mod ngx_sdk_guard;
pub(crate) mod ngx_super_resolution;
pub(crate) mod vulkan_extensions;
pub(crate) mod vulkan_resource;
