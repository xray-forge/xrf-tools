use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext};

use crate::feature::image::bloom::bloom_parameters::BloomParameters;
use crate::feature::image::bloom::bloom_stage::BloomStage;
use crate::frame::targets::view_targets::ViewTargets;
use crate::pass::build::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::build::pipeline_source::PipelineSource;

/// The engine's bloom (`phase_bloom`): the frame's high part built into the first bloom target, blurred across into the
/// second and down back into the first, which the present adds over the frame.
pub struct BloomPass {
  layout: wgpu::BindGroupLayout,
  /// `smp_rtlinear`: filtered, clamped to the edge.
  sampler: wgpu::Sampler,
  pipelines: PipelineSet<BloomPipelines>,
}

impl BloomPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = BloomParameters::create_layout(device);

    Ok(Self {
      pipelines: PipelineSet::new(source.cache, Self::create_pipelines(source, &layout)?),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("bloom"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self
      .pipelines
      .refresh(source.cache, || Self::create_pipelines(source, &self.layout))
    {
      log::error!("Bloom rejected, blooming with the last one: {error}");
    }
  }

  /// The sampler its draws read their source with, which their parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Draws one of its stages into the target the pass draws into.
  pub fn record(&self, context: &mut RasterContext<'_>, stage: BloomStage, parameters: &BloomParameters<'_>) {
    context.get_pass().set_pipeline(match stage {
      BloomStage::Build => &self.pipelines.build,
      BloomStage::Across | BloomStage::Down => &self.pipelines.filter,
    });
    context.bind(parameters);
    context.get_pass().draw(0..3, 0..1);
  }

  fn create_pipelines(source: PipelineSource<'_>, layout: &wgpu::BindGroupLayout) -> XrfResult<BloomPipelines> {
    let create = |fragment: &'static str| {
      create_fullscreen_pipeline(source, "frame/bloom", fragment, &[Some(layout)], ViewTargets::BLOOM)
    };

    Ok(BloomPipelines {
      build: create("fs_bloom_build")?,
      filter: create("fs_bloom_filter")?,
    })
  }
}

/// The engine bloom's pipelines: the build of the high part, and the blur across or down.
struct BloomPipelines {
  build: wgpu::RenderPipeline,
  filter: wgpu::RenderPipeline,
}
