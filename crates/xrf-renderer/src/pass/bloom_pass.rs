use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::bloom_parameters::BloomParameters;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::shader::shader_library::ShaderLibrary;

/// The engine's bloom (`phase_bloom`): the frame's high part built into the first bloom target, blurred across into the
/// second and down back into the first, which the present adds over the frame.
pub struct BloomPass {
  layout: wgpu::BindGroupLayout,
  /// `smp_rtlinear`: filtered, clamped to the edge.
  sampler: wgpu::Sampler,
  build: wgpu::RenderPipeline,
  filter: wgpu::RenderPipeline,
  generation: u64,
}

impl BloomPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = BloomParameters::create_layout(device);
    let (build, filter) = Self::create_pipelines(device, shaders, &layout)?;

    Ok(Self {
      build,
      filter,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("bloom"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.layout) {
        Ok((build, filter)) => (self.build, self.filter) = (build, filter),
        Err(error) => log::error!("Bloom rejected, blooming with the last one: {error}"),
      }
    }
  }

  /// The sampler its draws read their source with, which their parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// The bloom's three stages: built from the high target, blurred across, then down.
  pub const STAGES: [&'static str; 3] = ["bloom build", "bloom across", "bloom down"];

  /// Draws one of its stages into the target the pass draws into.
  pub fn record(&self, context: &mut RasterContext<'_>, stage: usize, parameters: &BloomParameters<'_>) {
    context
      .get_pass()
      .set_pipeline(if stage == 0 { &self.build } else { &self.filter });
    context.bind(parameters);
    context.get_pass().draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<(wgpu::RenderPipeline, wgpu::RenderPipeline)> {
    let create = |fragment: &str| {
      create_fullscreen_pipeline(
        device,
        shaders,
        "frame/bloom",
        fragment,
        &[Some(layout)],
        ViewTargets::BLOOM,
      )
    };

    Ok((create("fs_bloom_build")?, create("fs_bloom_filter")?))
  }
}
