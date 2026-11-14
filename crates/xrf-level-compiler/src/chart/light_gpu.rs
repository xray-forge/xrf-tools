use std::path::Path;

use glam::Vec3;
use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{BvhTracer, Tracer};
use xrf_renderer_core::{RayQueryAsk, RayQueryDevice, RenderGpu};

use crate::chart::light_chart::LightChart;
use crate::chart::light_chart_lighter::LightChartLighter;
use crate::chart::light_texel_point::LightTexelPoint;
use crate::chart::lightmap_layer::LightmapLayer;
use crate::chart::lit_chart::LitChart;
use crate::chart::lit_chart_next::LitChartNext;
use crate::light::light_color::LightColor;
use crate::light::light_lamp_rays::LightLampRays;
use crate::light::light_plan::LightPlan;
use crate::light::light_point::LightPoint;
use crate::light::light_point_variant::LightPointVariant;
use crate::light::light_ray_group::LightRayGroup;
use crate::light::light_sample_point::LightSamplePoint;
use crate::light::light_source::LightSource;
use crate::ray::gpu::ray_gpu_batch::RayGpuBatch;
use crate::ray::gpu::ray_gpu_tracer::RayGpuTracer;
use crate::ray::ray_gpu::RayGpu;
use crate::shadow::shadow_materials::ShadowMaterials;

/// The lightmaps' points lit with their shadow rays traced on the GPU by xrLC's rules: planned on the CPU's threads a
/// few hundred points a plan ([`LightPlan`]) with the lights reaching them, every light's rays a packet the GPU makes,
/// traced in batches as large as a dispatch takes, finished on the CPU's threads. Each face the device meets is tested
/// again as the CPU tests it, so a point's light is the CPU's but where the GPU's arithmetic parts from it in a last
/// bit: one point in a few hundred on a level of trees, a few steps of light, which settling a chart can carry further.
pub(crate) struct LightGpu {
  tracer: RayGpuTracer,
}

impl LightGpu {
  /// xrLC's `LightPoint`.
  const LIGHT_POINT: LightPoint = LightPoint {
    variant: LightPointVariant::Level,
  };

  /// How much wider than its points a plan's sphere is taken, in meters, when it selects the lights reaching them: far
  /// more than the rounding of a level's coordinates, so a light reaching a point at the sphere's edge stays.
  const REACH_MARGIN: f32 = 0.01;

  /// Rays traced together at most, unless the points run out first.
  const RAYS: usize = 1 << 23;

  /// Packets traced together at most: a dispatch's workgroups, less room for one more plan's.
  const PACKETS: usize = RayGpuTracer::MOST_WORKGROUPS / 2;

  /// The texel samples a chunk of charts is lit for together at most, each round of a chunk one batch or a few.
  const CHUNK_POINTS: usize = 1 << 21;

  /// The GPU of `gpu`, the high-performance one without it, with DXC's `dxc` on D3D12, the ray model of `tracer` and
  /// `shadows` on it.
  ///
  /// # Errors
  ///
  /// Returns an error when no GPU traces rays or the GPU refuses the ray model.
  pub(crate) fn start(
    dxc: Option<&Path>,
    gpu: Option<RayGpu>,
    tracer: &BvhTracer,
    shadows: &ShadowMaterials,
  ) -> XrfResult<Self> {
    let device: RayQueryDevice = RayQueryDevice::request(RayQueryAsk {
      dxc,
      gpu: gpu.map(RenderGpu::from),
    })?;

    Ok(Self {
      tracer: RayGpuTracer::new(device, tracer, shadows)?,
    })
  }

  /// The name of the GPU tracing.
  pub(crate) fn get_name(&self) -> &str {
    self.tracer.get_name()
  }

  /// Every chart of `charts` lit and bordered as [`LightChartLighter::light`] lights it, chunk by chunk: each chunk's
  /// charts' points lit together, each chart settled, those shrinking lit again together, each bordered.
  ///
  /// # Errors
  ///
  /// Returns an error when the GPU refuses a batch.
  pub(crate) fn light_charts<T: Tracer + Sync>(
    &self,
    lighter: &LightChartLighter<'_, T>,
    charts: &[LightChart],
  ) -> XrfResult<Vec<LitChart>> {
    let mut lit: Vec<LitChart> = Vec::with_capacity(charts.len());
    let mut start: usize = 0;

    while start < charts.len() {
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
          LitChartNext::Bordered(layer) => LightChartLighter::<T>::expand(layer),
          LitChartNext::Relit(..) => LightChartLighter::<T>::expand(
            seconds
              .next()
              .ok_or_else(|| XrfError::new_unexpected_error("A chart lit again has no second layer"))?,
          ),
        });
      }

      start = end;
    }

    Ok(lit)
  }

  /// Each of `charts` lit at its size of `sizes`, its points lit together.
  fn light_layers<T: Tracer + Sync>(
    &self,
    lighter: &LightChartLighter<'_, T>,
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
          LightChartLighter::<T>::assemble(*width, *height, chart, &colors[offset..offset + chart.len()])
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
    lights: &[Vec<LightSource>; 3],
    ray_faces: &[Option<u32>],
    points: &[LightTexelPoint],
  ) -> XrfResult<Vec<LightColor>> {
    let plans: Vec<LightPlan> = points
      .par_chunks(LightPlan::MOST_POINTS)
      .map(|chunk| {
        // A chunk's points lie together on a chart or a few: the lights reaching their sphere, the only ones giving any
        // of them light, a little wider so rounding keeps a light reaching its edge.
        let (low, high): (Vec3, Vec3) = chunk
          .iter()
          .fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(low, high), point| {
            (low.min(point.point), high.max(point.point))
          });
        let center: Vec3 = (low + high) * 0.5;
        let radius: f32 = center.distance(high) + Self::REACH_MARGIN;
        let selected: [Vec<u32>; 3] = lights.each_ref().map(|role| {
          (0..role.len() as u32)
            .filter(|light| role[*light as usize].reaches(center, radius))
            .collect()
        });
        let samples: Vec<LightSamplePoint> = chunk
          .iter()
          .map(|point| LightSamplePoint::new(point.point, point.normal))
          .collect();
        let skips: Vec<Option<u32>> = chunk
          .iter()
          .map(|point| ray_faces.get(point.face as usize).copied().flatten())
          .collect();

        LightPlan::new(
          Self::LIGHT_POINT,
          samples,
          skips,
          lights,
          selected,
          LightLampRays::Packet,
        )
      })
      .collect();
    let mut colors: Vec<LightColor> = Vec::with_capacity(points.len());
    let mut start: usize = 0;

    while start < plans.len() {
      let mut batch: RayGpuBatch = RayGpuBatch::default();
      let mut firsts: Vec<usize> = Vec::new();
      let mut end: usize = start;

      while end < plans.len() {
        let packets: usize = plans[end]
          .groups
          .iter()
          .filter(|group| !matches!(group, LightRayGroup::Single { .. }))
          .count();
        let singles: usize = plans[end].groups.len() - packets;
        let is_full: bool = batch.packets.len() + packets > Self::PACKETS
          || batch.singles.len() + singles > RayGpuTracer::MOST_SINGLES
          || batch.ray_count >= Self::RAYS;

        if is_full && end > start {
          break;
        }

        firsts.push(batch.push(&plans[end], |face| self.tracer.get_skip(face)));
        end += 1;
      }

      let transmittances: Vec<f32> = self.tracer.trace(&batch, LightPointVariant::Level)?;
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
