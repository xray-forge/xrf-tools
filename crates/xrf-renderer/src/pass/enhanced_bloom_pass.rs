use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::enhanced_bloom_parameters::EnhancedBloomParameters;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// The enhanced bloom, in place of the engine's: the finished scene blurred to half its size, its bright parts and
/// self-lit surfaces built from that, halved six times and doubled back with each halving added, then tonemapped into
/// what the present screens over the frame.
pub struct EnhancedBloomPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// `smp_rtlinear`: filtered, clamped to the edge.
  sampler: wgpu::Sampler,
  /// One a stage: the blur, the build, a halving, a doubling and the finish.
  pipelines: [wgpu::RenderPipeline; 5],
  generation: u64,
}

impl EnhancedBloomPass {
  /// The half-size blur's targets: eight bits a channel, as the engine's blur targets are.
  pub const BLURRED: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  /// Every other target: half floats, as the engine's bloom targets are.
  pub const BLOOM: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  /// The stages, by the pipeline each draws with.
  pub const BLUR: usize = 0;
  pub const BUILD: usize = 1;
  pub const HALVE: usize = 2;
  pub const DOUBLE: usize = 3;
  pub const FINISH: usize = 4;

  pub const BLUR_PASSES: [&'static str; 2] = ["enhanced bloom blur across", "enhanced bloom blur down"];
  pub const BUILD_PASS: &'static str = "enhanced bloom build";
  /// Each halving, by the size it draws: the first blurs at half the frame's size, the last draws a sixty-fourth.
  pub const HALVE_PASSES: [&'static str; 6] = [
    "enhanced bloom down 2",
    "enhanced bloom down 4",
    "enhanced bloom down 8",
    "enhanced bloom down 16",
    "enhanced bloom down 32",
    "enhanced bloom down 64",
  ];
  /// Each doubling, by the size it draws, from the thirty-second back to half the frame's size.
  pub const DOUBLE_PASSES: [&'static str; 5] = [
    "enhanced bloom up 32",
    "enhanced bloom up 16",
    "enhanced bloom up 8",
    "enhanced bloom up 4",
    "enhanced bloom up 2",
  ];
  pub const FINISH_PASS: &'static str = "enhanced bloom finish";

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = EnhancedBloomParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, (view_layout, &layout))?,
      view_layout: view_layout.clone(),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("enhanced bloom"),
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

      match Self::create_pipelines(device, shaders, (&self.view_layout, &self.layout)) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Enhanced bloom rejected, blooming with the last one: {error}"),
      }
    }
  }

  /// The sampler its stages read with, which their parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Draws one of its stages into the target the pass draws into.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    stage: usize,
    parameters: &EnhancedBloomParameters<'_>,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[stage]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    (view_layout, layout): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
  ) -> XrfResult<[wgpu::RenderPipeline; 5]> {
    let layouts: [Option<&wgpu::BindGroupLayout>; 2] = [Some(view_layout), Some(layout)];
    let create = |fragment: &str, format: wgpu::TextureFormat| {
      create_fullscreen_pipeline(device, shaders, "frame/enhanced_bloom", fragment, &layouts, format)
    };

    Ok([
      create("fs_bloom_blur", Self::BLURRED)?,
      create("fs_bloom_build", Self::BLOOM)?,
      create("fs_bloom_halve", Self::BLOOM)?,
      create("fs_bloom_double", Self::BLOOM)?,
      create("fs_bloom_finish", Self::BLOOM)?,
    ])
  }
}
