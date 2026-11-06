use xrf_error::XrfResult;
use xrf_renderer_core::{
  ComputeContext, ComputePipelineDescription, FrameGraph, GraphBindings, GraphTexture, PassParameters, PipelineSet,
};

use crate::feature::scene::pyramid::cover_pyramid::CoverPyramid;
use crate::feature::scene::pyramid::cover_pyramid_depth_parameters::CoverPyramidDepthParameters;
use crate::feature::scene::pyramid::cover_pyramid_level_parameters::CoverPyramidLevelParameters;
use crate::feature::scene::pyramid::pyramid_depth_parameters::PyramidDepthParameters;
use crate::feature::scene::pyramid::pyramid_level_parameters::PyramidLevelParameters;
use crate::feature::scene::pyramid::pyramid_reduction::PyramidReduction;
use crate::frame::targets::depth_pyramid::DepthPyramid;
use crate::pass::build::pipeline_source::PipelineSource;

/// Reduces a viewport's depth into a pyramid, a dispatch a level: to the farthest under each texel, or the nearest; and
/// a rain cover into a pyramid of both.
pub struct DepthPyramidPass {
  /// Reading the depth, then a level: the viewport's, then the cover's.
  layouts: [wgpu::BindGroupLayout; 4],
  /// Reducing the depth into the first level, then a level into the next: the farthest's pair, the nearest's, then the
  /// cover's.
  pipelines: PipelineSet<[wgpu::ComputePipeline; 6]>,
}

impl DepthPyramidPass {
  /// Invocations a reduction workgroup runs a side, as `shaders/frame/pyramid.wgsl` declares them.
  pub(crate) const WORKGROUP: u32 = 8;

  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layouts: [wgpu::BindGroupLayout; 4] = [
      PyramidDepthParameters::create_layout(device),
      PyramidLevelParameters::create_layout(device),
      CoverPyramidDepthParameters::create_layout(device),
      CoverPyramidLevelParameters::create_layout(device),
    ];

    Ok(Self {
      pipelines: PipelineSet::new(source.cache, Self::create_pipelines(source, &layouts)?),
      layouts,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self
      .pipelines
      .refresh(source.cache, || Self::create_pipelines(source, &self.layouts))
    {
      log::error!("Depth pyramid rejected, reducing with the last one: {error}");
    }
  }

  /// Declares `pyramid` reduced from the frame's `depth` as it stands then, as the compute pass `name`: each level its
  /// own resource, the depth reduced into the first and each into the next, keeping the depth `reduction` names.
  pub fn add_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    name: &'static str,
    (pyramid, depth): (&'a DepthPyramid, GraphTexture),
    reduction: PyramidReduction,
  ) {
    let label: &'static str = match reduction {
      PyramidReduction::Farthest => "depth pyramid level",
      PyramidReduction::Nearest => "nearest depth level",
    };
    let level_views: Vec<GraphTexture> = pyramid
      .level_views
      .iter()
      .map(|view| bindings.import_view(&mut *graph, label, view))
      .collect();
    let first: PyramidDepthParameters = PyramidDepthParameters {
      source_depth: depth,
      target_level: level_views[0],
    };
    let levels: Vec<PyramidLevelParameters> = level_views
      .windows(2)
      .map(|pair| PyramidLevelParameters {
        source_level: pair[0],
        target_level: pair[1],
      })
      .collect();

    levels
      .iter()
      .fold(graph.add_compute_pass(name).parameters(&first), |builder, level| {
        builder.parameters(level)
      })
      .record(move |context| self.record(context, pyramid, &first, &levels, reduction));
  }

  /// Reduces the depth into the first level, then each level into the next, a dispatch each, keeping the depth
  /// `reduction` names.
  pub fn record(
    &self,
    context: &mut ComputeContext<'_>,
    pyramid: &DepthPyramid,
    first: &PyramidDepthParameters,
    levels: &[PyramidLevelParameters],
    reduction: PyramidReduction,
  ) {
    let is_nearest: bool = reduction == PyramidReduction::Nearest;

    for level in 0..pyramid.levels as usize {
      let (width, height): (u32, u32) = pyramid.get_level_size(level as u32);

      match level {
        0 => context.bind(first),
        _ => context.bind(&levels[level - 1]),
      }

      let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

      pass.set_pipeline(&self.pipelines[usize::from(is_nearest) * 2 + usize::from(level > 0)]);
      pass.dispatch_workgroups(width.div_ceil(Self::WORKGROUP), height.div_ceil(Self::WORKGROUP), 1);
    }
  }

  /// Reduces a rain cover into its pyramid's first level, then each level into the next, a dispatch each.
  pub fn record_cover(
    &self,
    context: &mut ComputeContext<'_>,
    pyramid: &CoverPyramid,
    first: &CoverPyramidDepthParameters,
    levels: &[CoverPyramidLevelParameters],
  ) {
    for level in 0..pyramid.levels as usize {
      let size: u32 = pyramid.get_level_size(level as u32);

      match level {
        0 => context.bind(first),
        _ => context.bind(&levels[level - 1]),
      }

      let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

      pass.set_pipeline(&self.pipelines[4 + usize::from(level > 0)]);
      pass.dispatch_workgroups(size.div_ceil(Self::WORKGROUP), size.div_ceil(Self::WORKGROUP), 1);
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
      create("cs_reduce_depth", depth_layout)?,
      create("cs_reduce_level", level_layout)?,
      create("cs_copy_nearest", depth_layout)?,
      create("cs_reduce_nearest", level_layout)?,
      create("cs_reduce_cover", cover_layout)?,
      create("cs_reduce_cover_level", cover_level_layout)?,
    ])
  }
}
