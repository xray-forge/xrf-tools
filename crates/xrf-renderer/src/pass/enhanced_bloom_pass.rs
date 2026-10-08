use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext, RenderPipelineDescription};

use crate::pass::enhanced_bloom_parameters::EnhancedBloomParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

/// The enhanced bloom, in place of the engine's: the finished scene blurred to half its size, its bright parts and
/// self-lit surfaces built from that, halved six times and doubled back with each halving added, then tonemapped into
/// what the present screens over the frame. Its pipelines come from the pipeline cache, one a stage.
pub struct EnhancedBloomPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// `smp_rtlinear`: filtered, clamped to the edge.
  sampler: wgpu::Sampler,
}

impl EnhancedBloomPass {
  /// The half-size blur's targets: eight bits a channel, as the engine's blur targets are.
  pub const BLURRED: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  /// Every other target: half floats, as the engine's bloom targets are.
  pub const BLOOM: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  const MODULE: &'static str = "frame/enhanced_bloom";

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

  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout) -> Self {
    Self {
      layout: EnhancedBloomParameters::create_layout(device),
      view_layout: view_layout.clone(),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("enhanced bloom"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
    }
  }

  /// Its pipelines, one a stage by the stage's index: a triangle over the target, the view's bind group then the
  /// stage's, into the half-size blur's format or the bloom's.
  pub fn describe_pipelines(&self) -> [RenderPipelineDescription; 5] {
    let describe = |label: &'static str, fragment: &'static str, format: wgpu::TextureFormat| {
      RenderPipelineDescription::new(label, Self::MODULE, "vs_fullscreen")
        .with_fragment(fragment)
        .with_bind_group_layouts(vec![self.view_layout.clone(), self.layout.clone()])
        .with_target(wgpu::ColorTargetState::from(format))
    };

    [
      describe("enhanced bloom blur", "fs_bloom_blur", Self::BLURRED),
      describe("enhanced bloom build", "fs_bloom_build", Self::BLOOM),
      describe("enhanced bloom halve", "fs_bloom_halve", Self::BLOOM),
      describe("enhanced bloom double", "fs_bloom_double", Self::BLOOM),
      describe("enhanced bloom finish", "fs_bloom_finish", Self::BLOOM),
    ]
  }

  /// Its pipelines as `describe_pipelines` lists them, asked of the cache.
  ///
  /// # Errors
  ///
  /// Returns the first stage's error where a module cannot be composed or wgpu refuses it.
  pub fn get_pipelines(&self, source: PipelineSource<'_>) -> XrfResult<[wgpu::RenderPipeline; 5]> {
    let [blur, build, halve, double, finish] = self
      .describe_pipelines()
      .map(|description| source.get_render_pipeline(&description));

    Ok([blur?, build?, halve?, double?, finish?])
  }

  /// The sampler its stages read with, which their parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Draws one of its stages with the stage's `pipeline` into the target the pass draws into.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    pipeline: &wgpu::RenderPipeline,
    parameters: &EnhancedBloomParameters<'_>,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }
}
