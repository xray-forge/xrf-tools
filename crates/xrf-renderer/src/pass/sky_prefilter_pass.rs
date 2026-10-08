use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, PassParameters};

use crate::frame::sky_prefilter::SkyPrefilter;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::shader_pipelines::create_checked;
use crate::pass::sky_prefilter_parameters::SkyPrefilterParameters;

/// Invocations a workgroup runs a side, as `shaders/frame/sky_prefilter.wgsl` declares them.
const WORKGROUP: u32 = 8;

/// Fills the prefiltered sky cube, a mip a dispatch: the first from the sky's own cubes as the sky is drawn, each after
/// it by sampling the mips before it over the GGX lobe of its roughness.
pub struct SkyPrefilterPass {
  layout: wgpu::BindGroupLayout,
  pipeline: wgpu::ComputePipeline,
  /// What the first mip binds in place of the mips before it: one texel of nothing, a cube.
  empty: wgpu::TextureView,
  generation: u64,
}

impl SkyPrefilterPass {
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
  /// Directions a texel of a blurred mip samples over its lobe.
  pub const SAMPLES: u32 = 32;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, source: PipelineSource<'_>) -> XrfResult<Self> {
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
      pipeline: Self::create_pipeline(device, source, &layout)?,
      empty,
      generation: source.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(device, source, &self.layout) {
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
      .dispatch_workgroups(size.div_ceil(WORKGROUP), size.div_ceil(WORKGROUP), 6);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::ComputePipeline> {
    let module: wgpu::ShaderModule = source.get_module("frame/sky_prefilter")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("sky prefilter"),
      bind_group_layouts: &[Some(layout)],
      ..Default::default()
    });

    create_checked(device, "sky prefilter", || {
      device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("sky prefilter"),
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("prefilter"),
        compilation_options: Default::default(),
        cache: source.get_driver_cache(),
      })
    })
  }
}
