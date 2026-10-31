use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::context::gpu_requirements::can_trace;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::sun_trace_parameters::SunTraceParameters;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::view_binding::ViewBinding;

/// The sun's shadow traced at the frame's size: one ray a pixel towards a point of the sun's disc, traced against the
/// level's geometry by hardware ray query (`frame/sun_trace`), its visibility written for the sun's pass, where DLSS
/// ray reconstruction denoises it. Nothing traces on a device without ray queries.
pub struct SunTracePass {
  view_layout: wgpu::BindGroupLayout,
  /// Its own, the trace's and the texture cache's layouts, on a device with ray queries.
  layouts: Option<[wgpu::BindGroupLayout; 3]>,
  /// None on a device without ray queries.
  pipeline: Option<wgpu::RenderPipeline>,
  generation: u64,
}

impl SunTracePass {
  /// How much of the sun a pixel sees: one or none, a ray at a time.
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

  /// Where the pass binds the trace's group, and the texture cache's after it.
  const TRACE_GROUP: u32 = 3;
  const TEXTURE_GROUP: u32 = 4;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layouts: Option<[wgpu::BindGroupLayout; 3]> = can_trace(device).then(|| {
      [
        SunTraceParameters::create_layout(device),
        TraceParameters::create_layout(device),
        texture_layout.clone(),
      ]
    });

    Ok(Self {
      pipeline: Self::create_pipeline(source, view_layout, layouts.as_ref())?,
      view_layout: view_layout.clone(),
      layouts,
      generation: source.get_generation(),
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, &self.view_layout, self.layouts.as_ref()) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Sun trace rejected, tracing with the last one: {error}"),
      }
    }
  }

  /// Traces the frame's pixels towards the sun into the target the pass draws into.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &SunTraceParameters,
    trace: &TraceParameters<'_>,
    textures: &wgpu::BindGroup,
  ) {
    let Some(pipeline) = &self.pipeline else {
      return;
    };

    context.bind(parameters);
    context.bind(trace);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(Self::TEXTURE_GROUP, textures, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layouts: Option<&[wgpu::BindGroupLayout; 3]>,
  ) -> XrfResult<Option<wgpu::RenderPipeline>> {
    let Some([parameters_layout, trace_layout, texture_layout]) = layouts else {
      return Ok(None);
    };
    let mut groups: [Option<&wgpu::BindGroupLayout>; 5] = [None; 5];

    groups[0] = Some(view_layout);
    groups[1] = Some(parameters_layout);
    groups[Self::TRACE_GROUP as usize] = Some(trace_layout);
    groups[Self::TEXTURE_GROUP as usize] = Some(texture_layout);

    create_fullscreen_pipeline(source, "frame/sun_trace", "fs_sun_trace", &groups, Self::FORMAT).map(Some)
  }
}
