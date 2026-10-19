use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::context::gpu_requirements::can_trace;
use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline_with;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::rtao_permutation::RtaoPermutation;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::vbao_parameters::VbaoParameters;
use crate::pass::view_binding::ViewBinding;

/// RTAO at half the frame's size: a few short rays from each point, cosine-weighted about its normal, traced against
/// the level's geometry by hardware ray query (`frame/rtao`), their unblocked share written as the bitmask search
/// writes its visibility, for its accumulation and filter, under DLSS ray reconstruction too: a ray or few a point
/// multiplied straight into the frame are not a signal it reconstructs, and left dotted the frame. Nothing traces on a
/// device without ray queries.
pub struct RtaoPass {
  view_layout: wgpu::BindGroupLayout,
  /// The search's, the trace's and the texture cache's layouts, on a device with ray queries.
  layouts: Option<[wgpu::BindGroupLayout; 3]>,
  /// A pipeline a quality, on a device with ray queries.
  pipelines: Vec<wgpu::RenderPipeline>,
  generation: u64,
}

impl RtaoPass {
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
        VbaoParameters::create_layout(device),
        TraceParameters::create_layout(device),
        texture_layout.clone(),
      ]
    });

    Ok(Self {
      pipelines: Self::create_pipelines(source, view_layout, layouts.as_ref())?,
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

      match Self::create_pipelines(source, &self.view_layout, self.layouts.as_ref()) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("RTAO rejected, tracing with the last one: {error}"),
      }
    }
  }

  /// Traces the frame's points at `quality` into the target the pass draws into.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    quality: RenderAmbientOcclusionQuality,
    view: &ViewBinding,
    (parameters, trace, textures): (&VbaoParameters, &TraceParameters<'_>, &wgpu::BindGroup),
  ) {
    // One pipeline a quality in `RenderAmbientOcclusionQuality::ALL` order.
    let Some(pipeline) = self.pipelines.get(quality.get_index()) else {
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
  ) -> XrfResult<Vec<wgpu::RenderPipeline>> {
    let Some([search_layout, trace_layout, texture_layout]) = layouts else {
      return Ok(Vec::new());
    };
    let mut groups: [Option<&wgpu::BindGroupLayout>; 5] = [None; 5];

    groups[0] = Some(view_layout);
    groups[1] = Some(search_layout);
    groups[Self::TRACE_GROUP as usize] = Some(trace_layout);
    groups[Self::TEXTURE_GROUP as usize] = Some(texture_layout);

    let targets: [Option<wgpu::ColorTargetState>; 1] = [Some(ViewTargets::OCCLUSION.into())];

    RenderAmbientOcclusionQuality::ALL
      .into_iter()
      .map(|quality| {
        create_fullscreen_pipeline_with(
          source,
          (
            RtaoPermutation::MODULE,
            "fs_rtao",
            &RtaoPermutation {
              rays: quality.get_rtao_rays(),
            },
          ),
          &groups,
          &targets,
        )
      })
      .collect()
  }
}
