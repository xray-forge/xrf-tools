use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters};

use crate::frame::depth_pyramid::DepthPyramid;
use crate::pass::cover_pyramid_depth_parameters::CoverPyramidDepthParameters;
use crate::pass::cover_pyramid_level_parameters::CoverPyramidLevelParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::pyramid_depth_parameters::PyramidDepthParameters;
use crate::pass::pyramid_level_parameters::PyramidLevelParameters;
use crate::scene::level::cover_pyramid::CoverPyramid;

/// Invocations a reduction workgroup runs a side, as `shaders/frame/pyramid.wgsl` declares them.
const WORKGROUP: u32 = 8;

/// Reduces a viewport's depth into a pyramid, a dispatch a level: to the farthest under each texel, or the nearest; and
/// a rain cover into a pyramid of both.
pub struct DepthPyramidPass {
  /// Reading the depth, then a level: the viewport's, then the cover's.
  layouts: [wgpu::BindGroupLayout; 4],
  /// Reducing the depth into the first level, then a level into the next: the farthest's pair, the nearest's, then the
  /// cover's.
  pipelines: [wgpu::ComputePipeline; 6],
  generation: u64,
}

impl DepthPyramidPass {
  pub fn new(device: &wgpu::Device, source: PipelineSource<'_>) -> XrfResult<Self> {
    let layouts: [wgpu::BindGroupLayout; 4] = [
      PyramidDepthParameters::create_layout(device),
      PyramidLevelParameters::create_layout(device),
      CoverPyramidDepthParameters::create_layout(device),
      CoverPyramidLevelParameters::create_layout(device),
    ];

    Ok(Self {
      pipelines: Self::create_pipelines(source, &layouts)?,
      generation: source.get_generation(),
      layouts,
    })
  }

  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.layouts) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Depth pyramid rejected, reducing with the last one: {error}"),
      }
    }
  }

  /// Reduces the depth into the first level, then each level into the next, a dispatch each: to the nearest where
  /// `is_nearest`, the farthest otherwise.
  pub fn record(
    &self,
    context: &mut ComputeContext<'_>,
    pyramid: &DepthPyramid,
    (first, levels): (&PyramidDepthParameters, &[PyramidLevelParameters]),
    is_nearest: bool,
  ) {
    for level in 0..pyramid.levels as usize {
      let (width, height): (u32, u32) = pyramid.get_level_size(level as u32);

      match level {
        0 => context.bind(first),
        _ => context.bind(&levels[level - 1]),
      }

      let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

      pass.set_pipeline(&self.pipelines[usize::from(is_nearest) * 2 + usize::from(level > 0)]);
      pass.dispatch_workgroups(width.div_ceil(WORKGROUP), height.div_ceil(WORKGROUP), 1);
    }
  }

  /// Reduces a rain cover into its pyramid's first level, then each level into the next, a dispatch each.
  pub fn record_cover(
    &self,
    context: &mut ComputeContext<'_>,
    pyramid: &CoverPyramid,
    (first, levels): (&CoverPyramidDepthParameters, &[CoverPyramidLevelParameters]),
  ) {
    for level in 0..pyramid.levels as usize {
      let size: u32 = pyramid.get_level_size(level as u32);

      match level {
        0 => context.bind(first),
        _ => context.bind(&levels[level - 1]),
      }

      let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

      pass.set_pipeline(&self.pipelines[4 + usize::from(level > 0)]);
      pass.dispatch_workgroups(size.div_ceil(WORKGROUP), size.div_ceil(WORKGROUP), 1);
    }
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    [depth_layout, level_layout, cover_layout, cover_level_layout]: &[wgpu::BindGroupLayout; 4],
  ) -> XrfResult<[wgpu::ComputePipeline; 6]> {
    let create = |entry: &'static str, layout: &wgpu::BindGroupLayout| -> XrfResult<wgpu::ComputePipeline> {
      source.get_compute_pipeline(
        &ComputePipelineDescription::new(entry, "frame/pyramid", entry).with_bind_group_layouts(&[Some(layout)]),
      )
    };

    Ok([
      create("reduce_depth", depth_layout)?,
      create("reduce_level", level_layout)?,
      create("copy_nearest", depth_layout)?,
      create("reduce_nearest", level_layout)?,
      create("reduce_cover", cover_layout)?,
      create("reduce_cover_level", cover_level_layout)?,
    ])
  }
}
