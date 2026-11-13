#![expect(
  unsafe_code,
  reason = "opts into wgpu's experimental ray queries, which wgpu marks unsafe"
)]

use std::path::Path;

use xrf_error::{XrfError, XrfResult};

use crate::device::gpu_adapters::GpuAdapters;
use crate::device::ray_query_ask::RayQueryAsk;
use crate::device::render_gpu::RenderGpu;

/// A GPU device without a window that traces rays in compute shaders (`EXPERIMENTAL_RAY_QUERY`), for tools tracing
/// offline rather than drawing.
///
/// D3D12 compiles ray query shaders only through DXC, so it is tried only where a DXC library is given; Vulkan needs
/// none and is tried after it, or alone.
pub struct RayQueryDevice {
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  /// The adapter's name and backend, as a report names the GPU that traced: `NVIDIA GeForce RTX 5080 (D3D12)`.
  pub name: String,
}

impl RayQueryDevice {
  /// The features asked for: ray queries alone.
  pub const FEATURES: wgpu::Features = wgpu::Features::EXPERIMENTAL_RAY_QUERY;

  /// Starts the adapter that traces rays as `ask` asks: on D3D12 where it names DXC's `dxcompiler.dll` and the adapter
  /// traces there, else on Vulkan; the GPU it names where the backend has it tracing rays, else the high-performance
  /// one. Its own limits are asked for, so a tool can bind buffers as large as the device allows.
  ///
  /// # Errors
  ///
  /// Returns an error naming why each backend tried could not trace rays.
  pub fn request(ask: RayQueryAsk<'_>) -> XrfResult<Self> {
    let d3d12: Option<XrfResult<Self>> = ask
      .dxc
      .map(|dxc| Self::request_on(wgpu::Backends::DX12, Some(dxc), ask.gpu));

    match d3d12 {
      Some(Ok(device)) => Ok(device),
      Some(Err(d3d12)) => Self::request_on(wgpu::Backends::VULKAN, None, ask.gpu)
        .map_err(|vulkan| XrfError::new_unexpected_error(format!("{d3d12}; {vulkan}"))),
      None => Self::request_on(wgpu::Backends::VULKAN, None, ask.gpu),
    }
  }

  /// The adapter `gpu` names on `instance` where it traces rays; else, with a warning where one was named, the
  /// high-performance one.
  fn request_adapter(
    instance: &wgpu::Instance,
    backends: wgpu::Backends,
    label: &str,
    gpu: Option<RenderGpu>,
  ) -> XrfResult<wgpu::Adapter> {
    if let Some(gpu) = gpu {
      match GpuAdapters::find(instance, backends, gpu, |adapter| {
        adapter.features().contains(Self::FEATURES)
      }) {
        Some(adapter) => return Ok(adapter),
        None => log::warn!(
          "The chosen GPU ({:04x}:{:04x}) does not trace rays on {label}; tracing on the automatic one",
          gpu.vendor,
          gpu.device
        ),
      }
    }

    GpuAdapters::request_automatic(instance, false)
      .map_err(|error| XrfError::new_unexpected_error(format!("No {label} GPU to trace rays on: {error}")))
  }

  fn request_on(backends: wgpu::Backends, dxc: Option<&Path>, gpu: Option<RenderGpu>) -> XrfResult<Self> {
    let label: &str = if backends == wgpu::Backends::DX12 {
      "D3D12"
    } else {
      "Vulkan"
    };
    let mut descriptor: wgpu::InstanceDescriptor = wgpu::InstanceDescriptor::new_without_display_handle();

    descriptor.backends = backends;
    descriptor.flags = wgpu::InstanceFlags::from_build_config().with_env();

    if let Some(dxc) = dxc {
      descriptor.backend_options.dx12.shader_compiler = wgpu::Dx12Compiler::DynamicDxc {
        dxc_path: dxc.to_string_lossy().into_owned(),
      };
    }

    let instance: wgpu::Instance = wgpu::Instance::new(descriptor);
    let adapter: wgpu::Adapter = Self::request_adapter(&instance, backends, label, gpu)?;
    let name: String = format!("{} ({label})", adapter.get_info().name);

    if !adapter.features().contains(Self::FEATURES) {
      return Err(XrfError::new_unexpected_error(format!(
        "The GPU '{name}' cannot trace rays in a shader"
      )));
    }

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
      label: Some("xrf ray queries"),
      required_features: Self::FEATURES,
      required_limits: adapter.limits(),
      // SAFETY: only ray queries are asked for of the experimental features, and only where the adapter has them;
      // wgpu's own validation stands between them and the driver as for any other feature.
      experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
      ..Default::default()
    }))
    .map_err(|error| XrfError::new_unexpected_error(format!("The GPU '{name}' did not start: {error}")))?;

    Ok(Self { device, queue, name })
  }
}
