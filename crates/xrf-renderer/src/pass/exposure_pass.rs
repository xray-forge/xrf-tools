use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters};

use crate::pass::exposure_parameters::ExposureParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::scene::level::feature::exposure_feature::ExposureFeature;

/// Invocations a measuring workgroup runs, as `shaders/frame/exposure.wgsl` declares them.
const MEASURE_WORKGROUP: u32 = 64;

/// `phase_luminance`: measures the scene combine finished, then adapts the scale the next frame's tonemap reads.
pub struct ExposurePass {
  layout: wgpu::BindGroupLayout,
  /// Measuring, then adapting.
  pipelines: [wgpu::ComputePipeline; 2],
  generation: u64,
}

impl ExposurePass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = ExposureParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(source, &layout)?,
      generation: source.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Exposure rejected, adapting with the last one: {error}"),
      }
    }
  }

  pub fn record(&self, context: &mut ComputeContext<'_>, parameters: &ExposureParameters) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[0]);
    pass.dispatch_workgroups(
      (ExposureFeature::CELLS * ExposureFeature::CELLS).div_ceil(MEASURE_WORKGROUP),
      1,
      1,
    );
    pass.set_pipeline(&self.pipelines[1]);
    pass.dispatch_workgroups(1, 1, 1);
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::ComputePipeline; 2]> {
    let create = |entry: &'static str| -> XrfResult<wgpu::ComputePipeline> {
      source.get_compute_pipeline(
        &ComputePipelineDescription::new(entry, "frame/exposure", entry).with_bind_group_layouts(&[Some(layout)]),
      )
    };

    Ok([create("measure")?, create("adapt")?])
  }
}
