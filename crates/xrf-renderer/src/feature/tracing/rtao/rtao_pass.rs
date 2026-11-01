use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext};

use crate::context::gpu_requirements::can_trace;
use crate::feature::lighting::bitmask::vbao::vbao_parameters::VbaoParameters;
use crate::feature::tracing::view_trace::trace_parameters::TraceParameters;
use crate::frame::targets::view_targets::ViewTargets;
use crate::pass::build::fullscreen_pipeline::create_fullscreen_pipeline_into;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::pass::view::view_binding::ViewBinding;

/// RTAO at half the frame's size: a few short rays from each point, cosine-weighted about its normal, traced against
/// the level's geometry by hardware ray query (`frame/rtao`), their unblocked share written as the bitmask search
/// writes its visibility, for its accumulation and filter, under DLSS ray reconstruction too: a ray or few a point
/// multiplied straight into the frame are not a signal it reconstructs, and left dotted the frame. Nothing traces on a
/// device without ray queries.
pub struct RtaoPass {
  view_layout: wgpu::BindGroupLayout,
  /// The search's, the trace's and the texture cache's layouts, on a device with ray queries.
  layouts: Option<[wgpu::BindGroupLayout; 3]>,
  /// The trace, its rays the uniform's, on a device with ray queries.
  pipeline: PipelineSet<Option<wgpu::RenderPipeline>>,
}

impl RtaoPass {
  /// Where the pass binds the trace's group, and the texture cache's after it.
  const TRACE_GROUP: u32 = 3;
  const TEXTURE_GROUP: u32 = 4;
  /// The module its pipeline is made from.
  const MODULE: &'static str = "frame/rtao";

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
        VbaoParameters::create_layout(device),
        TraceParameters::create_layout(device),
        texture_layout.clone(),
      ]
    });

    Ok(Self {
      pipeline: PipelineSet::new(
        source.cache,
        Self::create_pipelines(source, view_layout, layouts.as_ref())?,
      ),
      view_layout: view_layout.clone(),
      layouts,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self.pipeline.refresh(source.cache, || {
      Self::create_pipelines(source, &self.view_layout, self.layouts.as_ref())
    }) {
      log::error!("RTAO rejected, tracing with the last one: {error}");
    }
  }

  /// Traces the frame's points into the target the pass draws into, as many rays a point as the uniform says.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &VbaoParameters,
    trace: &TraceParameters<'_>,
    textures: &wgpu::BindGroup,
  ) {
    let Some(pipeline) = &*self.pipeline else {
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

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layouts: Option<&[wgpu::BindGroupLayout; 3]>,
  ) -> XrfResult<Option<wgpu::RenderPipeline>> {
    let Some([search_layout, trace_layout, texture_layout]) = layouts else {
      return Ok(None);
    };
    let mut groups: [Option<&wgpu::BindGroupLayout>; 5] = [None; 5];

    groups[0] = Some(view_layout);
    groups[1] = Some(search_layout);
    groups[Self::TRACE_GROUP as usize] = Some(trace_layout);
    groups[Self::TEXTURE_GROUP as usize] = Some(texture_layout);

    let targets: [Option<wgpu::ColorTargetState>; 1] = [Some(ViewTargets::OCCLUSION.into())];

    create_fullscreen_pipeline_into(source, Self::MODULE, "fs_rtao", &groups, &targets).map(Some)
  }
}
