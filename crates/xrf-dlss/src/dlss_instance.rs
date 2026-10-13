use std::path::Path;

use crate::dlss::Dlss;
#[cfg(feature = "sdk")]
use crate::ngx::ngx_application::NgxApplication;

/// A wgpu instance which, built with the SDK, starts its devices with DLSS: on Vulkan carrying the instance extensions
/// NGX needs and opening devices with its device extensions, on D3D12 starting NGX on an NVIDIA GPU's device.
pub struct DlssInstance {
  pub instance: wgpu::Instance,
  /// The application as NGX knows it, where DLSS is asked for: on Vulkan, where NGX was hooked into the instance and
  /// its extensions are there.
  #[cfg(feature = "sdk")]
  application: Option<NgxApplication>,
}

impl DlssInstance {
  /// NVIDIA's PCI vendor identifier, the only vendor whose GPUs NGX starts on.
  #[cfg(feature = "sdk")]
  const NVIDIA: u32 = 0x10de;

  /// An instance of `descriptor`'s backends, ready for DLSS where `is_upscaled` and the backends are Vulkan or D3D12
  /// alone, loading its runtime library from `runtime` where the application ships one.
  pub fn new(descriptor: wgpu::InstanceDescriptor, is_upscaled: bool, runtime: Option<&Path>) -> Self {
    #[cfg(feature = "sdk")]
    if is_upscaled && descriptor.backends == wgpu::Backends::VULKAN {
      match Self::hook(&descriptor, NgxApplication::new(runtime)) {
        Ok(instance) => return instance,
        Err(error) => log::warn!("DLSS is unavailable, its Vulkan instance failing: {error}"),
      }
    }

    #[cfg(all(feature = "sdk", windows))]
    if is_upscaled && descriptor.backends == wgpu::Backends::DX12 {
      return Self {
        instance: wgpu::Instance::new(descriptor),
        application: Some(NgxApplication::new(runtime)),
      };
    }

    #[cfg(not(feature = "sdk"))]
    let _ = (is_upscaled, runtime);

    Self {
      instance: wgpu::Instance::new(descriptor),
      #[cfg(feature = "sdk")]
      application: None,
    }
  }

  /// Starts a device on `adapter`, one of this instance's, and DLSS on it where NGX is hooked in and the adapter has
  /// what it needs.
  ///
  /// # Errors
  ///
  /// Returns wgpu's error where the device cannot be started.
  pub fn request_device(
    &self,
    adapter: &wgpu::Adapter,
    descriptor: &wgpu::DeviceDescriptor<'_>,
  ) -> Result<(wgpu::Device, wgpu::Queue, Option<Dlss>), wgpu::RequestDeviceError> {
    #[cfg(feature = "sdk")]
    if let Some(application) = &self.application
      && adapter.get_info().backend == wgpu::Backend::Dx12
    {
      let (device, queue) = pollster::block_on(adapter.request_device(descriptor))?;
      let dlss: Option<Dlss> = (adapter.get_info().vendor == Self::NVIDIA)
        .then(|| Dlss::start(application, adapter, (&device, &queue)))
        .flatten();

      return Ok((device, queue, dlss));
    }

    #[cfg(feature = "sdk")]
    if let Some(application) = &self.application {
      match Self::open(application, adapter, descriptor) {
        Ok((device, queue, is_supported)) => {
          let dlss: Option<Dlss> = is_supported
            .then(|| Dlss::start(application, adapter, (&device, &queue)))
            .flatten();

          return Ok((device, queue, dlss));
        }
        Err(error) => log::warn!("DLSS is unavailable, its Vulkan device failing: {error}"),
      }
    }

    let (device, queue) = pollster::block_on(adapter.request_device(descriptor))?;

    Ok((device, queue, None))
  }

  /// A Vulkan instance with NGX's instance extensions, where the loader has them; one without NGX otherwise.
  #[cfg(feature = "sdk")]
  fn hook(descriptor: &wgpu::InstanceDescriptor, application: NgxApplication) -> xrf_error::XrfResult<Self> {
    use crate::ngx::vulkan_extensions::add_instance_extensions;

    let mut is_supported: xrf_error::XrfResult<bool> = Ok(false);
    let raw = unsafe {
      wgpu::hal::vulkan::Instance::init_with_callback(
        &wgpu::hal::InstanceDescriptor {
          name: "wgpu",
          flags: descriptor.flags,
          memory_budget_thresholds: descriptor.memory_budget_thresholds,
          backend_options: descriptor.backend_options.clone(),
          telemetry: None,
          display: None,
        },
        Some(Box::new(|mut args| {
          is_supported = add_instance_extensions(&application, &mut args);
        })),
      )
    }
    .map_err(|error| xrf_error::XrfError::new_unexpected_error(error.to_string()))?;
    let is_supported: bool = is_supported?;

    Ok(Self {
      instance: unsafe { wgpu::Instance::from_hal::<wgpu::hal::api::Vulkan>(raw) },
      application: is_supported.then_some(application),
    })
  }

  /// A Vulkan device on `adapter` with NGX's device extensions where the adapter has them, and whether it has.
  #[cfg(feature = "sdk")]
  fn open(
    application: &NgxApplication,
    adapter: &wgpu::Adapter,
    descriptor: &wgpu::DeviceDescriptor<'_>,
  ) -> xrf_error::XrfResult<(wgpu::Device, wgpu::Queue, bool)> {
    use crate::ngx::vulkan_extensions::add_device_extensions;

    let failed = |error: String| xrf_error::XrfError::new_unexpected_error(error);
    let hal = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }
      .ok_or_else(|| failed("the adapter is not a Vulkan one".to_string()))?;
    let mut is_supported: xrf_error::XrfResult<bool> = Ok(false);
    let opened = unsafe {
      hal.open_with_callback(
        descriptor.required_features,
        &descriptor.required_limits,
        &descriptor.memory_hints,
        Some(Box::new(|mut args| {
          is_supported = add_device_extensions(application, &mut args, &hal);
        })),
      )
    }
    .map_err(|error| failed(error.to_string()))?;
    let is_supported: bool = is_supported?;
    let (device, queue) = unsafe { adapter.create_device_from_hal::<wgpu::hal::api::Vulkan>(opened, descriptor) }
      .map_err(|error| failed(error.to_string()))?;

    Ok((device, queue, is_supported))
  }
}
