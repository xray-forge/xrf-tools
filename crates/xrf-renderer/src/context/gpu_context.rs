use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use xrf_error::{XrfError, XrfResult};

use crate::context::gpu_requirements::{OPTIONAL_FEATURES, REQUIRED_FEATURES, get_missing_features};
use crate::contract::render_backend::RenderBackend;
use crate::contract::render_backend_availability::RenderBackendAvailability;

/// The GPU the renderer draws with: one per process, owned by the render thread.
pub struct GpuContext {
  pub instance: wgpu::Instance,
  pub adapter: wgpu::Adapter,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub backend: RenderBackend,
  /// The adapter's name, as reports state it.
  pub adapter_name: String,
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

  /// Starts the GPU on the first backend of `RenderBackend::list_tried` that starts: the one asked for, else the first
  /// other that does, so a remembered backend this machine has lost falls back rather than failing.
  ///
  /// # Errors
  ///
  /// Returns an error, every backend's reason in it, when none starts.
  pub fn create_preferred(asked: Option<RenderBackend>) -> XrfResult<Self> {
    let mut problems: Vec<String> = Vec::new();

    for backend in RenderBackend::list_tried(asked) {
      match Self::create(backend) {
        Ok(context) => {
          if !problems.is_empty() {
            log::warn!("Renderer fell back to {}: {}", backend.get_label(), problems.join("; "));
          }

          return Ok(context);
        }
        Err(error) => problems.push(format!("{}: {error}", backend.get_label())),
      }
    }

    Err(XrfError::new_unexpected_error(format!(
      "No graphics backend starts: {}",
      problems.join("; ")
    )))
  }

  /// Whether the renderer can draw with a backend here: a hardware adapter of it with every feature it needs, without
  /// starting a device on it.
  pub fn probe(backend: RenderBackend) -> RenderBackendAvailability {
    match Self::request_adapter(backend, false).and_then(|(_, adapter)| Self::check_adapter(&adapter)) {
      Ok(info) => RenderBackendAvailability {
        backend,
        adapter: Some(info.name),
        problem: None,
      },
      Err(error) => RenderBackendAvailability {
        backend,
        adapter: None,
        problem: Some(error.to_string()),
      },
    }
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

  /// An instance of a backend and its high-performance adapter, a software one where `is_fallback`.
  fn request_adapter(backend: RenderBackend, is_fallback: bool) -> XrfResult<(wgpu::Instance, wgpu::Adapter)> {
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

    Ok((instance, adapter))
  }

  /// What an adapter is, where it has every feature the renderer needs.
  fn check_adapter(adapter: &wgpu::Adapter) -> XrfResult<wgpu::AdapterInfo> {
    let info: wgpu::AdapterInfo = adapter.get_info();
    let missing: wgpu::Features = get_missing_features(adapter.features());

    if missing.is_empty() {
      Ok(info)
    } else {
      Err(XrfError::new_unexpected_error(format!(
        "The GPU '{}' lacks what the renderer needs: {missing:?}",
        info.name
      )))
    }
  }

  fn create_with(backend: RenderBackend, is_fallback: bool) -> XrfResult<Self> {
    let (instance, adapter) = Self::request_adapter(backend, is_fallback)?;
    let info: wgpu::AdapterInfo = Self::check_adapter(&adapter)?;

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
