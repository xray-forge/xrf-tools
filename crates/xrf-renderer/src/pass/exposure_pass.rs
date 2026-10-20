use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters};

use crate::pass::exposure_histogram_parameters::ExposureHistogramParameters;
use crate::pass::exposure_parameters::ExposureParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::scene::level::feature::exposure_feature::ExposureFeature;

/// `phase_luminance`: measures the scene combine finished, then adapts the scale the next frame's tonemap reads; under
/// the physical light, a histogram of the scene's light, then the scale brought towards middle grey.
pub struct ExposurePass {
  layout: wgpu::BindGroupLayout,
  histogram_layout: wgpu::BindGroupLayout,
  /// Measuring, then adapting.
  pipelines: [wgpu::ComputePipeline; 2],
  /// Counting the histogram, then settling the scale.
  histogram: [wgpu::ComputePipeline; 2],
  generation: u64,
}

impl ExposurePass {
  /// Invocations a measuring workgroup runs, as `shaders/frame/exposure.wgsl` declares them.
  pub(crate) const MEASURE_WORKGROUP: u32 = 64;
  /// Invocations a counting workgroup runs each way, and texels each way a counted sample stands for, as
  /// `shaders/frame/exposure_histogram.wgsl` declares them.
  pub(crate) const HISTOGRAM_WORKGROUP: u32 = 16;
  pub(crate) const HISTOGRAM_STRIDE: u32 = 4;

  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = ExposureParameters::create_layout(device);
    let histogram_layout: wgpu::BindGroupLayout = ExposureHistogramParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(source, &layout)?,
      histogram: Self::create_histogram_pipelines(source, &histogram_layout)?,
      generation: source.get_generation(),
      layout,
      histogram_layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match (
        Self::create_pipelines(source, &self.layout),
        Self::create_histogram_pipelines(source, &self.histogram_layout),
      ) {
        (Ok(pipelines), Ok(histogram)) => (self.pipelines, self.histogram) = (pipelines, histogram),
        (Err(error), _) | (_, Err(error)) => log::error!("Exposure rejected, adapting with the last one: {error}"),
      }
    }
  }

  /// Measures the high part in cells, then adapts the exposure's scale towards what they read.
  pub fn record(&self, context: &mut ComputeContext<'_>, parameters: &ExposureParameters) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[0]);
    pass.dispatch_workgroups(
      (ExposureFeature::CELLS * ExposureFeature::CELLS).div_ceil(Self::MEASURE_WORKGROUP),
      1,
      1,
    );
    pass.set_pipeline(&self.pipelines[1]);
    pass.dispatch_workgroups(1, 1, 1);
  }

  /// Counts a `size` scene's light into the histogram, then settles the scale from it.
  pub fn record_histogram(
    &self,
    context: &mut ComputeContext<'_>,
    parameters: &ExposureHistogramParameters,
    (width, height): (u32, u32),
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();
    let span: u32 = Self::HISTOGRAM_WORKGROUP * Self::HISTOGRAM_STRIDE;

    pass.set_pipeline(&self.histogram[0]);
    pass.dispatch_workgroups(width.div_ceil(span), height.div_ceil(span), 1);
    pass.set_pipeline(&self.histogram[1]);
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

    Ok([create("cs_measure")?, create("cs_adapt")?])
  }

  fn create_histogram_pipelines(
    source: PipelineSource<'_>,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::ComputePipeline; 2]> {
    let create = |entry: &'static str| -> XrfResult<wgpu::ComputePipeline> {
      source.get_compute_pipeline(
        &ComputePipelineDescription::new(entry, "frame/exposure_histogram", entry)
          .with_bind_group_layouts(&[Some(layout)]),
      )
    };

    Ok([create("cs_bin")?, create("cs_settle")?])
  }
}
