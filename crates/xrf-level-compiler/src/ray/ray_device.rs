use xrf_error::XrfResult;

use crate::ray::ray_backend::RayBackend;
use crate::ray::ray_scene::RayScene;
use crate::ray::ray_tracing::RayTracing;

/// What traces a compiler's rays once started: a GPU holding the compiler's ray scene, or the CPU.
pub(crate) struct RayDevice {
  #[cfg(feature = "gpu")]
  gpu: Option<crate::ray::gpu::ray_gpu_tracer::RayGpuTracer>,
}

impl RayDevice {
  /// The device `tracing` names, the GPU started with `scene` on it where it traces there; `Auto` on the CPU where
  /// the GPU does not start, saying so with `what`, the rays traced.
  ///
  /// # Errors
  ///
  /// Returns an error for the GPU backend where this build has none, no GPU traces rays, or the GPU refuses the ray
  /// scene.
  pub(crate) fn start(tracing: &RayTracing<'_>, scene: RayScene<'_>, what: &str) -> XrfResult<Self> {
    if tracing.backend == RayBackend::Cpu {
      return Ok(Self::on_cpu());
    }

    match Self::start_gpu(tracing, scene, what) {
      Ok(device) => Ok(device),
      Err(error) if tracing.backend == RayBackend::Auto => {
        log::info!("Tracing {what} on the CPU: {error}");

        Ok(Self::on_cpu())
      }
      Err(error) => Err(error),
    }
  }

  /// The GPU `tracing` names, the high-performance one without it, with `scene` on it.
  #[cfg(feature = "gpu")]
  fn start_gpu(tracing: &RayTracing<'_>, scene: RayScene<'_>, _what: &str) -> XrfResult<Self> {
    use xrf_renderer_core::{RayQueryAsk, RayQueryDevice, RenderGpu};

    use crate::ray::gpu::ray_gpu_tracer::RayGpuTracer;

    let device: RayQueryDevice = RayQueryDevice::request(RayQueryAsk {
      dxc: tracing.dxc,
      gpu: tracing.gpu.map(RenderGpu::from),
    })?;

    Ok(Self {
      gpu: Some(RayGpuTracer::new(device, scene.tracer, scene.shadows)?),
    })
  }

  /// Refused: this build has no GPU backend.
  #[cfg(not(feature = "gpu"))]
  fn start_gpu(tracing: &RayTracing<'_>, scene: RayScene<'_>, what: &str) -> XrfResult<Self> {
    let _ = (tracing, scene);

    Err(xrf_error::XrfError::new_invalid_error(format!(
      "This build traces {what} on the CPU alone; its GPU backend is the `gpu` feature"
    )))
  }

  /// The CPU.
  fn on_cpu() -> Self {
    Self {
      #[cfg(feature = "gpu")]
      gpu: None,
    }
  }

  /// The name of the GPU tracing, none for the CPU.
  pub(crate) fn get_name(&self) -> Option<String> {
    #[cfg(feature = "gpu")]
    {
      self.gpu.as_ref().map(|gpu| gpu.get_name().to_owned())
    }

    #[cfg(not(feature = "gpu"))]
    {
      None
    }
  }

  /// The GPU's tracer, none for the CPU.
  #[cfg(feature = "gpu")]
  pub(crate) fn get_gpu(&self) -> Option<&crate::ray::gpu::ray_gpu_tracer::RayGpuTracer> {
    self.gpu.as_ref()
  }
}
