use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use xrf_error::{XrfError, XrfResult};

use crate::context::gpu_requirements::{OPTIONAL_FEATURES, REQUIRED_FEATURES, get_missing_features};
use crate::context::render_backend::RenderBackend;

/// The GPU the renderer draws with: one per process, owned by the render thread.
pub struct GpuContext {
  pub instance: wgpu::Instance,
  pub adapter: wgpu::Adapter,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub backend: RenderBackend,
  /// The adapter's name, as reports state it.
  pub adapter_name: String,
  /// Whether timestamp queries are available for pass timings.
  pub is_timed: bool,
  lost: Arc<AtomicBool>,
}

impl GpuContext {
  /// Starts the GPU on a hardware adapter of the backend asked for.
  ///
  /// # Errors
  ///
  /// Returns an error when no adapter of that backend exists, when it lacks a required feature, or when its device
  /// cannot be created.
  pub fn create(backend: RenderBackend) -> XrfResult<Self> {
    Self::create_with(backend, false)
  }

  /// Starts the GPU for drawing without a window, falling back to a software adapter (WARP under D3D12) where no
  /// hardware one exists, which is what a test machine without a GPU has.
  ///
  /// # Errors
  ///
  /// Returns an error when neither a hardware nor a software adapter can be had with the required features.
  pub fn create_headless(backend: RenderBackend) -> XrfResult<Self> {
    Self::create_with(backend, false).or_else(|_| Self::create_with(backend, true))
  }

  /// Whether the device was lost, after which nothing it made draws again.
  pub fn is_lost(&self) -> bool {
    self.lost.load(Ordering::Acquire)
  }

  fn create_with(backend: RenderBackend, is_fallback: bool) -> XrfResult<Self> {
    let mut descriptor: wgpu::InstanceDescriptor = wgpu::InstanceDescriptor::new_without_display_handle();

    descriptor.backends = backend.to_backends();
    // Validation in debug builds only, so a release frame pays nothing for it; `WGPU_VALIDATION`, `WGPU_DEBUG` and the
    // rest of wgpu's switches override it either way.
    descriptor.flags = wgpu::InstanceFlags::from_build_config().with_env();

    let instance: wgpu::Instance = wgpu::Instance::new(descriptor);
    let adapter: wgpu::Adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::HighPerformance,
      compatible_surface: None,
      force_fallback_adapter: is_fallback,
      apply_limit_buckets: false,
    }))
    .map_err(|error| XrfError::new_unexpected_error(format!("No {} adapter: {error}", backend.get_label())))?;

    let info: wgpu::AdapterInfo = adapter.get_info();
    let missing: wgpu::Features = get_missing_features(adapter.features());

    if !missing.is_empty() {
      return Err(XrfError::new_unexpected_error(format!(
        "The GPU '{}' lacks what the renderer needs: {missing:?}",
        info.name
      )));
    }

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
      label: Some("xrf-renderer"),
      required_features: REQUIRED_FEATURES | (OPTIONAL_FEATURES & adapter.features()),
      required_limits: adapter.limits(),
      ..Default::default()
    }))
    .map_err(|error| XrfError::new_unexpected_error(format!("Failed to start the GPU '{}': {error}", info.name)))?;

    let lost: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let lost_flag: Arc<AtomicBool> = Arc::clone(&lost);

    device.set_device_lost_callback(move |reason, message| {
      log::error!("GPU device lost ({reason:?}): {message}");
      lost_flag.store(true, Ordering::Release);
    });
    device.on_uncaptured_error(Arc::new(|error: wgpu::Error| {
      log::error!("GPU error: {error}");
    }));

    log::info!(
      "Renderer started on '{}' ({}, {:?}, driver {} {})",
      info.name,
      backend.get_label(),
      info.device_type,
      info.driver,
      info.driver_info
    );

    Ok(Self {
      is_timed: adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY),
      adapter_name: info.name,
      instance,
      adapter,
      device,
      queue,
      backend,
      lost,
    })
  }
}
