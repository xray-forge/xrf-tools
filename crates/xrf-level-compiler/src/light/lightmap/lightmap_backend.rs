use xrf_error::XrfResult;
use xrf_job::parallel::prelude::*;
use xrf_job::{JobHandle, JobScope};

use crate::light::implicit::implicit_lighter::ImplicitLighter;
use crate::light::implicit::implicit_map::ImplicitMap;
use crate::light::lightmap::chart::light_chart::LightChart;
use crate::light::lightmap::lit::light_chart_lighter::LightChartLighter;
use crate::light::lightmap::lit::lit_chart::LitChart;
use crate::ray::ray_device::RayDevice;
use crate::ray::ray_tracing::RayTracing;

/// What traces the lightmaps' and implicit maps' shadow rays: the CPU, each chart on a thread, every light first tested
/// against the face that last stopped it; or a GPU (`LightmapGpu`, with the `gpu` feature), every point's rays in
/// batches. Started from a lighter's own ray scene, it lights with that lighter alone.
pub(crate) struct LightmapBackend {
  device: RayDevice,
}

impl LightmapBackend {
  /// What a job reports the lightmaps under: the charts lit.
  pub(crate) const PROGRESS_CHARTS: &'static str = "lightmap charts";

  /// The backend `tracing` names, started with `lighter`'s ray scene where it traces on a GPU; `Auto` on the CPU where
  /// the GPU does not start.
  ///
  /// # Errors
  ///
  /// Returns an error for the GPU backend where this build has none, no GPU traces rays, or the GPU refuses the ray
  /// model.
  pub(crate) fn start(tracing: &RayTracing<'_>, lighter: &LightChartLighter<'_>) -> XrfResult<Self> {
    Ok(Self {
      device: RayDevice::start(tracing, lighter.rays, "the lightmaps' shadow rays")?,
    })
  }

  /// The name of the GPU tracing, none for the CPU.
  pub(crate) fn get_name(&self) -> Option<String> {
    self.device.get_name()
  }

  /// Every one of `charts` lit and bordered by `lighter`, the one the backend was started with, each counted to `job`
  /// as it is done.
  ///
  /// # Errors
  ///
  /// Returns an error when the GPU refuses a batch, or the job is cancelled.
  pub(crate) fn light_charts(
    &self,
    lighter: &LightChartLighter<'_>,
    charts: &[LightChart],
    job: &JobHandle,
  ) -> XrfResult<Vec<LitChart>> {
    let scope: JobScope = job.enter(Self::PROGRESS_CHARTS, Some(charts.len() as u64));

    #[cfg(feature = "gpu")]
    if let Some(tracer) = self.device.get_gpu() {
      return crate::light::lightmap::lightmap_gpu::LightmapGpu { tracer }.light_charts(lighter, charts, job, &scope);
    }

    charts
      .par_iter()
      .map(|chart| {
        job.check_cancelled()?;

        let lit: LitChart = lighter.light(chart);

        scope.advance();

        Ok(lit)
      })
      .collect()
  }

  /// The implicit map of texture `name` by `implicit` ([`ImplicitLighter::light`]), whose lighter is the one the
  /// backend was started with.
  ///
  /// # Errors
  ///
  /// Returns an error when the GPU refuses a batch, or the job is cancelled.
  pub(crate) fn light_implicit(
    &self,
    implicit: &ImplicitLighter<'_>,
    name: String,
    base: Vec<[u8; 4]>,
    width: u32,
    height: u32,
    faces: &[u32],
  ) -> XrfResult<ImplicitMap> {
    #[cfg(feature = "gpu")]
    if let Some(tracer) = self.device.get_gpu() {
      return implicit.light_on_gpu(
        &crate::light::lightmap::lightmap_gpu::LightmapGpu { tracer },
        name,
        base,
        width,
        height,
        faces,
      );
    }

    implicit.light(name, base, width, height, faces)
  }
}
