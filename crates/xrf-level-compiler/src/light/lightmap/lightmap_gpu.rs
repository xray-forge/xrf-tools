use xrf_error::{XrfError, XrfResult};
use xrf_job::parallel::prelude::*;
use xrf_job::{JobHandle, JobScope};
use xrf_math::glam::Vec3;

use crate::light::lightmap::chart::light_chart::LightChart;
use crate::light::lightmap::lit::light_chart_lighter::LightChartLighter;
use crate::light::lightmap::lit::light_texel_point::LightTexelPoint;
use crate::light::lightmap::lit::lightmap_layer::LightmapLayer;
use crate::light::lightmap::lit::lit_chart::LitChart;
use crate::light::lightmap::lit::lit_chart_next::LitChartNext;
use crate::light::point::light_color::LightColor;
use crate::light::point::light_plan::LightPlan;
use crate::light::point::light_point::LightPoint;
use crate::light::point::light_point_variant::LightPointVariant;
use crate::light::point::light_sample_point::LightSamplePoint;
use crate::light::source::light_roles::LightRoles;
use crate::light::source::light_sources::LightSources;
use crate::ray::gpu::ray_gpu_batch::RayGpuBatch;
use crate::ray::gpu::ray_gpu_tracer::RayGpuTracer;
use crate::ray::ray_lamp_mode::RayLampMode;

/// The lightmaps' points lit by xrLC's rules, their shadow rays traced on a started GPU. The CPU's threads plan a few
/// hundred points at a time ([`LightPlan`]) with the lights reaching them and finish their light; the GPU makes each
/// light's rays as a packet and traces them in dispatch-sized batches.
///
/// Each face the device meets is retested as the CPU tests it, so a point's light is the CPU's but where the GPU's
/// arithmetic parts in a last bit: one point in a few hundred on a level of trees, a few steps of light, which
/// settling a chart can carry further.
pub(crate) struct LightmapGpu<'a> {
  /// The GPU holding the lighter's ray scene.
  pub(crate) tracer: &'a RayGpuTracer,
}

impl LightmapGpu<'_> {
  /// xrLC's `LightPoint`.
  const LIGHT_POINT: LightPoint = LightPoint {
    variant: LightPointVariant::Level,
  };

  /// How much wider than its points a plan's sphere is taken when selecting the lights reaching them, in meters: far
  /// more than a level's coordinate rounding, so a light reaching a point at the edge stays.
  const REACH_MARGIN: f32 = 0.01;

  /// The texel samples a chunk of charts is lit for together at most, each round of a chunk one batch or a few.
  const CHUNK_POINTS: usize = 1 << 21;

  /// Every chart of `charts` lit and bordered as [`LightChartLighter::light`] lights it, chunk by chunk: each chunk's
  /// charts' points lit together, each chart settled, those shrinking lit again together, each bordered, the chunk
  /// counted to `scope`.
  ///
  /// # Errors
  ///
  /// Returns an error when the GPU refuses a batch, or `job` is cancelled.
  pub(crate) fn light_charts(
    &self,
    lighter: &LightChartLighter<'_>,
    charts: &[LightChart],
    job: &JobHandle,
    scope: &JobScope,
  ) -> XrfResult<Vec<LitChart>> {
    let mut lit: Vec<LitChart> = Vec::with_capacity(charts.len());
    let mut start: usize = 0;

    while start < charts.len() {
      job.check_cancelled()?;

      let mut end: usize = start;
      let mut points: usize = 0;

      while end < charts.len() && (end == start || points < Self::CHUNK_POINTS) {
        points += (charts[end].width * charts[end].height) as usize * lighter.scene.jitter_samples.max(1) as usize;
        end += 1;
      }

      let chunk: &[LightChart] = &charts[start..end];
      let sizes: Vec<(u32, u32)> = chunk.iter().map(|chart| (chart.width, chart.height)).collect();
      let firsts: Vec<_> = self.light_layers(lighter, chunk, &sizes)?;
      let nexts: Vec<LitChartNext> = firsts.into_par_iter().map(|layer| lighter.settle(layer)).collect();
      let (relit, sizes): (Vec<usize>, Vec<(u32, u32)>) = nexts
        .iter()
        .enumerate()
        .filter_map(|(index, next)| match next {
          LitChartNext::Relit(width, height) => Some((index, (*width, *height))),
          _ => None,
        })
        .unzip();
      let relit_charts: Vec<LightChart> = relit.iter().map(|index| chunk[*index].clone()).collect();
      let mut seconds = self.light_layers(lighter, &relit_charts, &sizes)?.into_iter();

      for next in nexts {
        lit.push(match next {
          LitChartNext::Lit(done) => done,
          LitChartNext::Bordered(layer) => LightChartLighter::expand(layer),
          LitChartNext::Relit(..) => LightChartLighter::expand(
            seconds
              .next()
              .ok_or_else(|| XrfError::new_unexpected_error("A chart lit again has no second layer"))?,
          ),
        });
      }

      scope.advance_by(chunk.len() as u64);
      start = end;
    }

    Ok(lit)
  }

  /// Each of `charts` lit at its size of `sizes`, its points lit together.
  fn light_layers(
    &self,
    lighter: &LightChartLighter<'_>,
    charts: &[LightChart],
    sizes: &[(u32, u32)],
  ) -> XrfResult<Vec<LightmapLayer>> {
    let points: Vec<Vec<LightTexelPoint>> = charts
      .par_iter()
      .zip(sizes)
      .map(|(chart, (width, height))| lighter.get_points(chart, *width, *height))
      .collect();
    let all: Vec<LightTexelPoint> = points.iter().flatten().copied().collect();
    let colors: Vec<LightColor> = self.light_points(lighter.lights, lighter.ray_faces, &all)?;
    let mut offsets: Vec<usize> = Vec::with_capacity(points.len());
    let mut offset: usize = 0;

    for chart in &points {
      offsets.push(offset);
      offset += chart.len();
    }

    Ok(
      points
        .par_iter()
        .zip(sizes)
        .zip(offsets)
        .map(|((chart, (width, height)), offset)| {
          LightChartLighter::assemble(*width, *height, chart, &colors[offset..offset + chart.len()])
        })
        .collect(),
    )
  }

  /// Each of `points`' light from `lights`, the face it lies on passed (its index in the ray model by `ray_faces`), in
  /// their order.
  ///
  /// # Errors
  ///
  /// Returns an error when the GPU refuses a batch.
  pub(crate) fn light_points(
    &self,
    lights: &LightSources,
    ray_faces: &[Option<u32>],
    points: &[LightTexelPoint],
  ) -> XrfResult<Vec<LightColor>> {
    let plans: Vec<LightPlan> = points
      .par_chunks(LightPlan::MOST_POINTS)
      .map(|chunk| {
        // A chunk's points lie together on a chart or a few, so only the lights reaching their sphere give them light.
        let (low, high): (Vec3, Vec3) = chunk
          .iter()
          .fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(low, high), point| {
            (low.min(point.point), high.max(point.point))
          });
        let center: Vec3 = (low + high) * 0.5;
        let radius: f32 = center.distance(high) + Self::REACH_MARGIN;
        let selected: LightRoles<Vec<u32>> = lights.select_reaching(center, radius);
        let samples: Vec<LightSamplePoint> = chunk
          .iter()
          .map(|point| LightSamplePoint::new(point.point, point.normal))
          .collect();
        let skips: Vec<Option<u32>> = chunk
          .iter()
          .map(|point| ray_faces.get(point.face as usize).copied().flatten())
          .collect();

        LightPlan::new(Self::LIGHT_POINT, samples, skips, lights, selected, RayLampMode::Packet)
      })
      .collect();
    let mut colors: Vec<LightColor> = Vec::with_capacity(points.len());
    let mut start: usize = 0;

    while start < plans.len() {
      let mut batch: RayGpuBatch = RayGpuBatch::default();
      let mut firsts: Vec<usize> = Vec::new();
      let mut end: usize = start;

      while end < plans.len() {
        if batch.is_full_for(&plans[end]) && end > start {
          break;
        }

        firsts.push(batch.push(&plans[end], |face| self.tracer.get_skip(face)));
        end += 1;
      }

      let transmittances: Vec<f32> = self.tracer.trace(&batch, Self::LIGHT_POINT.variant.get_rules())?;
      let finished: Vec<Vec<LightColor>> = plans[start..end]
        .par_iter()
        .zip(firsts)
        .map(|(plan, first)| plan.get_colors(lights, &transmittances[first..first + plan.ray_count]))
        .collect();

      colors.extend(finished.into_iter().flatten());
      start = end;
    }

    Ok(colors)
  }
}
