use std::path::Path;

use rayon::prelude::*;
use xrf_error::XrfResult;
use xrf_geometry::{BvhTracer, Tracer};

use crate::chart::implicit_lighter::ImplicitLighter;
use crate::chart::implicit_map::ImplicitMap;
use crate::chart::light_chart::LightChart;
use crate::chart::light_chart_lighter::LightChartLighter;
use crate::chart::lit_chart::LitChart;
use crate::ray::ray_backend::RayBackend;
use crate::ray::ray_gpu::RayGpu;
use crate::shadow::shadow_materials::ShadowMaterials;

/// What traces the lightmaps' and implicit maps' shadow rays: the CPU, each chart on a thread, every light first tested
/// against the face that last stopped it; or a GPU ([`crate::chart::light_gpu::LightGpu`]), every point's rays in
/// batches.
pub(crate) struct LightmapBackend {
  #[cfg(feature = "gpu")]
  gpu: Option<crate::chart::light_gpu::LightGpu>,
}

impl LightmapBackend {
  /// The backend `backend` names, the GPU `gpu` (the high-performance one without it) started with DXC's `dxc` and the
  /// ray model of `tracer` and `shadows` where it traces there; `Auto` on the CPU where the GPU does not start.
  ///
  /// # Errors
  ///
  /// Returns an error for the GPU backend where this build has none, no GPU traces rays, or the GPU refuses the ray
  /// model.
  pub(crate) fn start(
    backend: RayBackend,
    dxc: Option<&Path>,
    gpu: Option<RayGpu>,
    tracer: &BvhTracer,
    shadows: &ShadowMaterials,
  ) -> XrfResult<Self> {
    if backend == RayBackend::Cpu {
      return Ok(Self::on_cpu());
    }

    #[cfg(feature = "gpu")]
    {
      match crate::chart::light_gpu::LightGpu::start(dxc, gpu, tracer, shadows) {
        Ok(gpu) => Ok(Self { gpu: Some(gpu) }),
        Err(error) if backend == RayBackend::Auto => {
          log::info!("Lighting the lightmaps on the CPU: {error}");

          Ok(Self::on_cpu())
        }
        Err(error) => Err(error),
      }
    }

    #[cfg(not(feature = "gpu"))]
    {
      let _ = (dxc, gpu, tracer, shadows);
      let error: xrf_error::XrfError = xrf_error::XrfError::new_invalid_error(
        "This build lights lightmaps on the CPU alone; its GPU backend is the `gpu` feature",
      );

      if backend == RayBackend::Auto {
        log::info!("Lighting the lightmaps on the CPU: {error}");

        Ok(Self::on_cpu())
      } else {
        Err(error)
      }
    }
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

  /// Every one of `charts` lit and bordered by `lighter`.
  ///
  /// # Errors
  ///
  /// Returns an error when the GPU refuses a batch.
  pub(crate) fn light_charts<T: Tracer + Sync>(
    &self,
    lighter: &LightChartLighter<'_, T>,
    charts: &[LightChart],
  ) -> XrfResult<Vec<LitChart>> {
    #[cfg(feature = "gpu")]
    if let Some(gpu) = &self.gpu {
      return gpu.light_charts(lighter, charts);
    }

    Ok(charts.par_iter().map(|chart| lighter.light(chart)).collect())
  }

  /// The implicit map of texture `name` by `implicit` ([`ImplicitLighter::light`]).
  ///
  /// # Errors
  ///
  /// Returns an error when the GPU refuses a batch.
  pub(crate) fn light_implicit<T: Tracer + Sync>(
    &self,
    implicit: &ImplicitLighter<'_, T>,
    name: String,
    base: Vec<[u8; 4]>,
    width: u32,
    height: u32,
    faces: &[u32],
  ) -> XrfResult<ImplicitMap> {
    #[cfg(feature = "gpu")]
    if let Some(gpu) = &self.gpu {
      return implicit.light_on_gpu(gpu, name, base, width, height, faces);
    }

    implicit.light(name, base, width, height, faces)
  }
}
