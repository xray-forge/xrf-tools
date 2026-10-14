//! NVIDIA DLSS super resolution and ray reconstruction for the renderer's views, through NGX on wgpu's Vulkan and D3D12
//! backends.
//!
//! Built without the `sdk` feature it offers the same types, and no device has DLSS. The NGX glue is derived from
//! dlss_wgpu (MIT OR Apache-2.0, `LICENSE-dlss_wgpu-*`).

mod dlss;
mod dlss_inputs;
mod dlss_instance;
mod dlss_preset;
mod dlss_quality;
mod dlss_ray_inputs;
mod dlss_ray_reconstructor;
mod dlss_upscaler;
#[cfg(feature = "sdk")]
mod ngx;

pub use crate::dlss::Dlss;
pub use crate::dlss_inputs::DlssInputs;
pub use crate::dlss_instance::DlssInstance;
pub use crate::dlss_preset::DlssPreset;
pub use crate::dlss_quality::DlssQuality;
pub use crate::dlss_ray_inputs::DlssRayInputs;
pub use crate::dlss_ray_reconstructor::DlssRayReconstructor;
pub use crate::dlss_upscaler::DlssUpscaler;
