//! The graphics APIs NGX runs on: which one a device is, its textures as NGX names them, and Vulkan's extensions.

pub(crate) mod ngx_api;
#[cfg(windows)]
pub(crate) mod ngx_d3d12_resource;
pub(crate) mod ngx_vulkan_extensions;
pub(crate) mod ngx_vulkan_resource;
