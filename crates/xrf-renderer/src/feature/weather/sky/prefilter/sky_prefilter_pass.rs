use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters, PipelineSet};

use crate::feature::weather::sky::prefilter::sky_prefilter::SkyPrefilter;
use crate::feature::weather::sky::prefilter::sky_prefilter_parameters::SkyPrefilterParameters;
use crate::pass::build::pipeline_source::PipelineSource;

/// Fills the prefiltered sky cube, a mip a dispatch: the first from the sky's own cubes as the sky is drawn, each after
/// it by sampling the mips before it over the GGX lobe of its roughness.
pub(crate) struct SkyPrefilterPass {
  layout: wgpu::BindGroupLayout,
  pipeline: PipelineSet<wgpu::ComputePipeline>,
  /// What the first mip binds in place of the mips before it: one texel of nothing, a cube.
  empty: wgpu::TextureView,
}

impl SkyPrefilterPass {
  /// Invocations a workgroup runs a side, as `shaders/frame/sky_prefilter.wgsl` declares them.
  pub(crate) const WORKGROUP: u32 = 8;

  /// Each mip's pass, by its mip.
  pub(crate) const PASSES: [&'static str; SkyPrefilter::MOST_LEVELS as usize] = [
    "sky prefilter 0",
    "sky prefilter 1",
    "sky prefilter 2",
    "sky prefilter 3",
    "sky prefilter 4",
    "sky prefilter 5",
    "sky prefilter 6",
    "sky prefilter 7",
    "sky prefilter 8",
  ];
  /// Directions a texel of a blurred mip samples over its lobe.
  pub(crate) const SAMPLES: u32 = 32;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub(crate) fn new(queue: &wgpu::Queue, source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = SkyPrefilterParameters::create_layout(device);
    let empty: wgpu::TextureView = device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("sky prefilter none"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 6,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: SkyPrefilter::FORMAT,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        bytemuck::cast_slice(&[0u16; 4 * 6]),
      )
      .create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
      });

    Ok(Self {
      pipeline: PipelineSet::new(source.cache, Self::create_pipeline(source, &layout)?),
      empty,
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub(crate) fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self
      .pipeline
      .refresh(source.cache, || Self::create_pipeline(source, &self.layout))
    {
      log::error!("Sky prefilter rejected, filling with the last one: {error}");
    }
  }

  /// The cube the first mip binds in place of the mips before it.
  pub(crate) fn get_empty(&self) -> &wgpu::TextureView {
    &self.empty
  }

  /// Fills one mip, `size` texels a side, six faces.
  pub(crate) fn record(&self, context: &mut ComputeContext<'_>, size: u32, parameters: &SkyPrefilterParameters<'_>) {
    context.bind(parameters);
    context.get_pass().set_pipeline(&self.pipeline);
    context
      .get_pass()
      .dispatch_workgroups(size.div_ceil(Self::WORKGROUP), size.div_ceil(Self::WORKGROUP), 6);
  }

  fn create_pipeline(source: PipelineSource<'_>, layout: &wgpu::BindGroupLayout) -> XrfResult<wgpu::ComputePipeline> {
    source.get_compute_pipeline(
      &ComputePipelineDescription::new("sky prefilter", "frame/sky_prefilter", "cs_prefilter")
        .with_bind_group_layouts(&[Some(layout)]),
    )
  }
}
