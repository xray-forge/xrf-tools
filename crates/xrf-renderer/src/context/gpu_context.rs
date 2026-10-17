use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use xrf_dlss::{Dlss, DlssInstance};
use xrf_env_flags::{EnvFlag, EnvFlags};
use xrf_error::{XrfError, XrfResult};

use crate::context::gpu_requirements::{
  OPTIONAL_FEATURES, REQUIRED_FEATURES, can_trace, get_missing_features, get_ray_query_features,
};
use crate::contract::render_backend::RenderBackend;
use crate::contract::render_backend_availability::RenderBackendAvailability;
use crate::contract::render_gpu::RenderGpu;
use crate::contract::render_gpu_description::RenderGpuDescription;

/// The GPU the renderer draws with: one per process, owned by the render thread.
pub struct GpuContext {
  pub instance: wgpu::Instance,
  pub adapter: wgpu::Adapter,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub backend: RenderBackend,
  /// The adapter's name, as reports state it.
  pub adapter_name: String,
  /// The GPU drawn on.
  pub gpu: RenderGpu,
  /// DLSS where the build and the GPU both have it: an RTX GPU, on Vulkan or D3D12.
  pub dlss: Option<Dlss>,
  lost: Arc<AtomicBool>,
}

impl GpuContext {
  /// Where the renderer's bundle keeps DXC's library, which D3D12 compiles shaders with for shader model 6 (ray
  /// queries among what it brings); FXC, which Windows carries, stops at 5.1.
  pub const DXC_LIBRARY: &'static str = "dxc/dxcompiler.dll";
  /// Where the renderer's bundle keeps DLSS's runtime library, where a build with DLSS ships it.
  pub const DLSS_DIRECTORY: &'static str = "dlss";
  /// Bytes of each memory block the allocator places resources in, a larger resource taking a block of its own: small,
  /// so the holes textures streaming in and out leave in the blocks hold little.
  pub const MEMORY_BLOCK: u64 = 16 * 1024 * 1024;

  /// Starts the GPU on a hardware adapter of a backend: the GPU `gpu` names where the backend has it, else the one the
  /// backend picks automatically. D3D12 compiles with the DXC library at the first of `libraries` where it loads, and
  /// Vulkan loads DLSS's runtime library from the directory at the second where the build has DLSS.
  ///
  /// # Errors
  ///
  /// Returns an error when no adapter of that backend exists, when it lacks a required feature, or when its device
  /// cannot be created.
  pub fn create(
    backend: RenderBackend,
    gpu: Option<RenderGpu>,
    libraries: (Option<&Path>, Option<&Path>),
  ) -> XrfResult<Self> {
    Self::create_with(backend, gpu, false, libraries)
  }

  /// Starts the GPU on the first backend of `RenderBackend::list_tried` that starts: the one asked for, else the first
  /// other that does, so a remembered backend this machine has lost falls back rather than failing.
  ///
  /// # Errors
  ///
  /// Returns an error, every backend's reason in it, when none starts.
  pub fn create_preferred(
    asked: Option<RenderBackend>,
    gpu: Option<RenderGpu>,
    libraries: (Option<&Path>, Option<&Path>),
  ) -> XrfResult<Self> {
    let mut problems: Vec<String> = Vec::new();

    for backend in RenderBackend::list_tried(asked) {
      match Self::create(backend, gpu, libraries) {
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

  /// Whether the renderer can draw with a backend here: every hardware GPU of it with the features the renderer needs,
  /// without starting a device on any.
  pub fn probe(backend: RenderBackend) -> RenderBackendAvailability {
    let is_forced: bool = RenderBackend::get_forced() == Some(backend);

    match Self::list_gpus(backend) {
      Ok(gpus) => RenderBackendAvailability {
        backend,
        gpus,
        problem: None,
        is_forced,
      },
      Err(error) => RenderBackendAvailability {
        backend,
        gpus: Vec::new(),
        problem: Some(error.to_string()),
        is_forced,
      },
    }
  }

  /// Starts the GPU for drawing without a window, falling back to a software adapter (WARP under D3D12) where no
  /// hardware one exists, which is what a test machine without a GPU has.
  ///
  /// # Errors
  ///
  /// Returns an error when neither a hardware nor a software adapter can be had with the required features.
  pub fn create_headless(backend: RenderBackend, dxc: Option<&Path>) -> XrfResult<Self> {
    Self::create_with(backend, None, false, (dxc, None))
      .or_else(|_| Self::create_with(backend, None, true, (dxc, None)))
  }

  /// Whether the device was lost, after which nothing it made draws again.
  pub fn is_lost(&self) -> bool {
    self.lost.load(Ordering::Acquire)
  }

  /// Every hardware GPU of a backend with the features the renderer needs, the one it picks automatically first.
  fn list_gpus(backend: RenderBackend) -> XrfResult<Vec<RenderGpuDescription>> {
    let instance: DlssInstance = Self::create_instance(backend, false, (None, None));
    let automatic: Option<wgpu::Adapter> = Self::request_automatic(&instance.instance, backend, false).ok();
    let listed: Vec<wgpu::Adapter> = pollster::block_on(instance.instance.enumerate_adapters(backend.to_backends()));
    let mut gpus: Vec<RenderGpuDescription> = Vec::new();
    let mut problems: Vec<String> = Vec::new();

    for adapter in automatic.into_iter().chain(listed) {
      let info: wgpu::AdapterInfo = adapter.get_info();

      if info.device_type == wgpu::DeviceType::Cpu || gpus.iter().any(|it| it.gpu == RenderGpu::of(&info)) {
        continue;
      }

      match Self::check_adapter(&adapter) {
        Ok(info) => gpus.push(RenderGpuDescription::describe(&info)),
        Err(error) => problems.push(error.to_string()),
      }
    }

    match (gpus.is_empty(), problems.is_empty()) {
      (false, _) => Ok(gpus),
      (true, true) => Err(XrfError::new_unexpected_error(format!(
        "No {} adapter",
        backend.get_label()
      ))),
      (true, false) => Err(XrfError::new_unexpected_error(problems.join("; "))),
    }
  }

  /// Creates an instance of a backend. D3D12 compiles with the DXC library at `dxc`, or FXC if none is given; with
  /// `is_upscaled` the instance is ready for DLSS, loading its runtime from `dlss` where the application ships one.
  fn create_instance(
    backend: RenderBackend,
    is_upscaled: bool,
    (dxc, dlss): (Option<&Path>, Option<&Path>),
  ) -> DlssInstance {
    let mut descriptor: wgpu::InstanceDescriptor = wgpu::InstanceDescriptor::new_without_display_handle();

    descriptor.backends = backend.to_backends();
    descriptor.backend_options.dx12.shader_compiler = match dxc {
      Some(path) => wgpu::Dx12Compiler::DynamicDxc {
        dxc_path: path.to_string_lossy().into_owned(),
      },
      None => wgpu::Dx12Compiler::Fxc,
    };
    // Validation in debug builds only, so a release frame pays nothing for it; `WGPU_VALIDATION`, `WGPU_DEBUG` and the
    // rest of wgpu's switches override it either way.
    descriptor.flags = Self::read_instance_flags();

    DlssInstance::new(descriptor, is_upscaled, dlss)
  }

  /// The build's instance flags with each of wgpu's switches set in the environment applied, read as any xrf switch.
  fn read_instance_flags() -> wgpu::InstanceFlags {
    Self::apply_instance_switches(wgpu::InstanceFlags::from_build_config(), EnvFlag::read_switch)
  }

  /// `flags` with each wgpu switch `read` answers for turned on or off as it answers.
  pub(crate) fn apply_instance_switches(
    mut flags: wgpu::InstanceFlags,
    read: impl Fn(&EnvFlag) -> Option<bool>,
  ) -> wgpu::InstanceFlags {
    for (switch, flag) in [
      (EnvFlags::WGPU_VALIDATION, wgpu::InstanceFlags::VALIDATION),
      (EnvFlags::WGPU_DEBUG, wgpu::InstanceFlags::DEBUG),
      (
        EnvFlags::WGPU_DISCARD_HAL_LABELS,
        wgpu::InstanceFlags::DISCARD_HAL_LABELS,
      ),
      (
        EnvFlags::WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
        wgpu::InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
      ),
      (
        EnvFlags::WGPU_GPU_BASED_VALIDATION,
        wgpu::InstanceFlags::GPU_BASED_VALIDATION,
      ),
      (
        EnvFlags::WGPU_VALIDATION_INDIRECT_CALL,
        wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL,
      ),
      (
        EnvFlags::WGPU_STRICT_WEBGPU_COMPLIANCE,
        wgpu::InstanceFlags::STRICT_WEBGPU_COMPLIANCE,
      ),
    ] {
      if let Some(is_on) = read(&switch) {
        flags.set(flag, is_on);
      }
    }

    flags
  }

  /// The adapter the backend picks automatically: its high-performance one, or a software one if `is_fallback`.
  fn request_automatic(
    instance: &wgpu::Instance,
    backend: RenderBackend,
    is_fallback: bool,
  ) -> XrfResult<wgpu::Adapter> {
    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::HighPerformance,
      compatible_surface: None,
      force_fallback_adapter: is_fallback,
      apply_limit_buckets: false,
    }))
    .map_err(|error| XrfError::new_unexpected_error(format!("No {} adapter: {error}", backend.get_label())))
  }

  /// Creates an instance of a backend and picks its adapter: the hardware GPU `gpu` names if the backend has it with
  /// every required feature, otherwise the automatic one (a software one if `is_fallback`).
  fn request_adapter(
    backend: RenderBackend,
    gpu: Option<RenderGpu>,
    (is_fallback, is_upscaled): (bool, bool),
    libraries: (Option<&Path>, Option<&Path>),
  ) -> XrfResult<(DlssInstance, wgpu::Adapter)> {
    let instance: DlssInstance = Self::create_instance(backend, is_upscaled, libraries);

    if let Some(asked) = gpu.filter(|_| !is_fallback) {
      let listed: Vec<wgpu::Adapter> = pollster::block_on(instance.instance.enumerate_adapters(backend.to_backends()));
      let chosen: Option<wgpu::Adapter> = listed.into_iter().find(|adapter| {
        let info: wgpu::AdapterInfo = adapter.get_info();

        info.device_type != wgpu::DeviceType::Cpu
          && RenderGpu::of(&info) == asked
          && Self::check_adapter(adapter).is_ok()
      });

      match chosen {
        Some(adapter) => return Ok((instance, adapter)),
        None => log::warn!(
          "The chosen GPU ({:04x}:{:04x}) cannot draw on {}; drawing on the automatic one",
          asked.vendor,
          asked.device,
          backend.get_label()
        ),
      }
    }

    let adapter: wgpu::Adapter = Self::request_automatic(&instance.instance, backend, is_fallback)?;

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

  fn create_with(
    backend: RenderBackend,
    gpu: Option<RenderGpu>,
    is_fallback: bool,
    (dxc, dlss): (Option<&Path>, Option<&Path>),
  ) -> XrfResult<Self> {
    let dxc: Option<&Path> = dxc.filter(|_| backend == RenderBackend::D3d12);
    // A DXC library that does not load fails the instance's D3D12 backend, so FXC is tried in its place.
    let upscaled: (bool, bool) = (is_fallback, !is_fallback);
    let (instance, adapter, shader_compiler) = match Self::request_adapter(backend, gpu, upscaled, (dxc, dlss)) {
      Ok((instance, adapter)) => (instance, adapter, dxc.map_or("FXC", |_| "DXC")),
      Err(error) if dxc.is_some() => {
        log::warn!("D3D12 compiles with FXC, the DXC library not loading: {error}");

        let (instance, adapter) = Self::request_adapter(backend, gpu, upscaled, (None, dlss))?;

        (instance, adapter, "FXC")
      }
      Err(error) => return Err(error),
    };
    let shader_compiler: Option<&'static str> = (backend == RenderBackend::D3d12).then_some(shader_compiler);
    let info: wgpu::AdapterInfo = Self::check_adapter(&adapter)?;

    let ray_query: wgpu::Features = get_ray_query_features(adapter.features());
    let (device, queue, dlss) = instance
      .request_device(
        &adapter,
        &wgpu::DeviceDescriptor {
          label: Some("xrf-renderer"),
          required_features: REQUIRED_FEATURES | (OPTIONAL_FEATURES & adapter.features()) | ray_query,
          required_limits: adapter.limits(),
          // SAFETY: only ray queries are asked for of the experimental features, and only where the adapter has them;
          // wgpu's own validation stands between them and the driver as for any other feature.
          experimental_features: if ray_query.is_empty() {
            wgpu::ExperimentalFeatures::disabled()
          } else {
            unsafe { wgpu::ExperimentalFeatures::enabled() }
          },
          memory_hints: wgpu::MemoryHints::Manual {
            suballocated_device_memory_block_size: Self::MEMORY_BLOCK..Self::MEMORY_BLOCK,
          },
          ..Default::default()
        },
      )
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
      "Renderer started on '{}' ({}{}, {:?}, driver {}), {}{}",
      info.name,
      backend.get_label(),
      shader_compiler.map(|it| format!(" with {it}")).unwrap_or_default(),
      info.device_type,
      RenderGpuDescription::describe(&info)
        .driver
        .as_deref()
        .unwrap_or("unknown"),
      if can_trace(&device) {
        "tracing rays"
      } else {
        "without ray queries"
      },
      if dlss.is_some() { ", with DLSS" } else { "" }
    );

    Ok(Self {
      gpu: RenderGpu::of(&info),
      adapter_name: info.name,
      instance: instance.instance,
      adapter,
      device,
      queue,
      backend,
      dlss,
      lost,
    })
  }
}
