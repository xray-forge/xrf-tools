use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters};

use crate::pass::pipeline_source::PipelineSource;
use crate::pass::sky_prefilter_parameters::SkyPrefilterParameters;
use crate::scene::level::feature::sky_prefilter::SkyPrefilter;

/// Fills the prefiltered sky cube, a mip a dispatch: the first from the sky's own cubes as the sky is drawn, each after
/// it by sampling the mips before it over the GGX lobe of its roughness; and the sky alone's cube the same, its first
/// mip the sky where the whole cube's first holds no surroundings.
pub struct SkyPrefilterPass {
  layout: wgpu::BindGroupLayout,
  pipeline: wgpu::ComputePipeline,
  /// What the first mip binds in place of the mips before it: one texel of nothing, a cube.
  empty: wgpu::TextureView,
  generation: u64,
}

impl SkyPrefilterPass {
  /// Invocations a workgroup runs a side, as `shaders/frame/sky_prefilter.wgsl` declares them.
  pub(crate) const WORKGROUP: u32 = 8;

  /// Each mip's pass, by its mip.
  pub const PASSES: [&'static str; SkyPrefilter::MOST_LEVELS as usize] = [
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
  /// Each mip's pass of the sky alone's cube (`SkyPrefilter::alone`), by its mip.
  pub const ALONE_PASSES: [&'static str; SkyPrefilter::MOST_LEVELS as usize] = [
    "sky prefilter alone 0",
    "sky prefilter alone 1",
    "sky prefilter alone 2",
    "sky prefilter alone 3",
    "sky prefilter alone 4",
    "sky prefilter alone 5",
    "sky prefilter alone 6",
    "sky prefilter alone 7",
    "sky prefilter alone 8",
  ];
  /// Every pass a refill runs, both cubes', which the scheduler times it by.
  pub const FILL_PASSES: [&'static str; 2 * SkyPrefilter::MOST_LEVELS as usize] = [
    Self::PASSES[0],
    Self::PASSES[1],
    Self::PASSES[2],
    Self::PASSES[3],
    Self::PASSES[4],
    Self::PASSES[5],
    Self::PASSES[6],
    Self::PASSES[7],
    Self::PASSES[8],
    Self::ALONE_PASSES[0],
    Self::ALONE_PASSES[1],
    Self::ALONE_PASSES[2],
    Self::ALONE_PASSES[3],
    Self::ALONE_PASSES[4],
    Self::ALONE_PASSES[5],
    Self::ALONE_PASSES[6],
    Self::ALONE_PASSES[7],
    Self::ALONE_PASSES[8],
  ];
  /// Directions a texel of a blurred mip samples over its lobe.
  pub const SAMPLES: u32 = 32;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(queue: &wgpu::Queue, source: PipelineSource<'_>) -> XrfResult<Self> {
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
      pipeline: Self::create_pipeline(source, &layout)?,
      empty,
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Sky prefilter rejected, filling with the last one: {error}"),
      }
    }
  }

  /// The cube the first mip binds in place of the mips before it.
  pub fn get_empty(&self) -> &wgpu::TextureView {
    &self.empty
  }

  /// Fills one mip, `size` texels a side, six faces.
  pub fn record(&self, context: &mut ComputeContext<'_>, size: u32, parameters: &SkyPrefilterParameters<'_>) {
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
