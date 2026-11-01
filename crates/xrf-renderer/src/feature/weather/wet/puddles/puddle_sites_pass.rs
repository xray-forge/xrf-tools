use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters, PipelineSet};

use crate::feature::weather::wet::puddles::lowest_heights_parameters::LowestHeightsParameters;
use crate::feature::weather::wet::puddles::puddle_keep_parameters::PuddleKeepParameters;
use crate::feature::weather::wet::puddles::puddle_sites_parameters::PuddleSitesParameters;
use crate::pass::pipeline_source::PipelineSource;

/// Places the puddles on the level's surface seen from overhead: its lowest heights, then a site a cell, a dispatch
/// each, where it is drawn; and each frame, which sites hold a puddle and how big.
pub struct PuddleSitesPass {
  layouts: [wgpu::BindGroupLayout; 3],
  /// The lowest heights, the sites, and the puddles.
  pipelines: PipelineSet<[wgpu::ComputePipeline; 3]>,
}

impl PuddleSitesPass {
  /// Invocations a workgroup runs a side, as `shaders/frame/puddle_sites.wgsl` declares them.
  pub(crate) const WORKGROUP: u32 = 8;

  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layouts: [wgpu::BindGroupLayout; 3] = [
      LowestHeightsParameters::create_layout(device),
      PuddleSitesParameters::create_layout(device),
      PuddleKeepParameters::create_layout(device),
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
      log::error!("Puddle sites rejected, placing with the last ones: {error}");
    }
  }

  /// Takes the lowest heights over a map `size` texels a side, then places a site in each of `cells` a side.
  pub fn record(
    &self,
    context: &mut ComputeContext<'_>,
    size: u32,
    cells: u32,
    lowest: &LowestHeightsParameters,
    sites: &PuddleSitesParameters,
  ) {
    context.bind(lowest);
    context.get_pass().set_pipeline(&self.pipelines[0]);
    context
      .get_pass()
      .dispatch_workgroups(size.div_ceil(Self::WORKGROUP), size.div_ceil(Self::WORKGROUP), 1);
    context.bind(sites);
    context.get_pass().set_pipeline(&self.pipelines[1]);
    context
      .get_pass()
      .dispatch_workgroups(cells.div_ceil(Self::WORKGROUP), cells.div_ceil(Self::WORKGROUP), 1);
  }

  /// Decides each of `cells` a side's puddle this frame.
  pub fn record_keep(&self, context: &mut ComputeContext<'_>, cells: u32, keep: &PuddleKeepParameters<'_>) {
    context.bind(keep);
    context.get_pass().set_pipeline(&self.pipelines[2]);
    context
      .get_pass()
      .dispatch_workgroups(cells.div_ceil(Self::WORKGROUP), cells.div_ceil(Self::WORKGROUP), 1);
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    [heights, sites, keep]: &[wgpu::BindGroupLayout; 3],
  ) -> XrfResult<[wgpu::ComputePipeline; 3]> {
    let create = |entry: &'static str, layout: &wgpu::BindGroupLayout| -> XrfResult<wgpu::ComputePipeline> {
      source.get_compute_pipeline(
        &ComputePipelineDescription::new(entry, "frame/puddle_sites", entry).with_bind_group_layouts(&[Some(layout)]),
      )
    };

    Ok([
      create("cs_surface_lowest", heights)?,
      create("cs_surface_sites", sites)?,
      create("cs_puddle_keep", keep)?,
    ])
  }
}
