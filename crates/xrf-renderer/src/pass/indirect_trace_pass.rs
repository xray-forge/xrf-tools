use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::context::gpu_requirements::can_trace;
use crate::pass::indirect_trace_parameters::IndirectTraceParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::shader_pipelines::create_checked;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::vbao_pass::VbaoPass;
use crate::pass::view_binding::ViewBinding;

/// The indirect light traced at half the frame's size where DLSS ray reconstruction denoises the frame
/// (`frame/indirect_trace`): one ray from each point, cosine-weighted about its normal, traced against the level's
/// geometry by hardware ray query and lit where it meets something as the reflections light a hit, written as the
/// screen-space bounce's filter writes its light, which combine and the debug view read alike. Nothing traces on a device
/// without ray queries.
pub struct IndirectTracePass {
  view_layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  /// Its own, the trace's and the texture cache's layouts, on a device with ray queries.
  layouts: Option<[wgpu::BindGroupLayout; 3]>,
  pipeline: Option<wgpu::RenderPipeline>,
  generation: u64,
}

impl IndirectTracePass {
  /// Metres each point's ray went, none where it met nothing.
  pub const HITS: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;

  /// Where the pass binds the sky's group, the trace's and the texture cache's.
  const SKY_GROUP: u32 = 2;
  const TRACE_GROUP: u32 = 3;
  const TEXTURE_GROUP: u32 = 4;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layouts: Option<[wgpu::BindGroupLayout; 3]> = can_trace(device).then(|| {
      [
        IndirectTraceParameters::create_layout(device),
        TraceParameters::create_layout(device),
        texture_layout.clone(),
      ]
    });

    Ok(Self {
      pipeline: Self::create_pipeline(source, (view_layout, sky_layout), layouts.as_ref())?,
      view_layout: view_layout.clone(),
      sky_layout: sky_layout.clone(),
      layouts,
      generation: source.get_generation(),
    })
  }

  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, (&self.view_layout, &self.sky_layout), self.layouts.as_ref()) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Indirect trace rejected, tracing with the last one: {error}"),
      }
    }
  }

  /// Whether it traces: a device with ray queries.
  pub fn is_traced(&self) -> bool {
    self.layouts.is_some()
  }

  /// Traces the frame's points into the light and hit targets the pass draws into.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    (parameters, sky): (&IndirectTraceParameters<'_>, &SkyParameters<'_>),
    (trace, textures): (&TraceParameters<'_>, &wgpu::BindGroup),
  ) {
    let Some(pipeline) = &self.pipeline else {
      return;
    };

    context.bind(parameters);
    context.bind(sky);
    context.bind(trace);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(Self::TEXTURE_GROUP, textures, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    source: PipelineSource<'_>,
    (view_layout, sky_layout): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
    layouts: Option<&[wgpu::BindGroupLayout; 3]>,
  ) -> XrfResult<Option<wgpu::RenderPipeline>> {
    let device: &wgpu::Device = source.device;

    let Some([own_layout, trace_layout, texture_layout]) = layouts else {
      return Ok(None);
    };
    let module: wgpu::ShaderModule = source.get_module("frame/indirect_trace")?;
    let mut groups: [Option<&wgpu::BindGroupLayout>; 5] = [None; 5];

    groups[0] = Some(view_layout);
    groups[1] = Some(own_layout);
    groups[Self::SKY_GROUP as usize] = Some(sky_layout);
    groups[Self::TRACE_GROUP as usize] = Some(trace_layout);
    groups[Self::TEXTURE_GROUP as usize] = Some(texture_layout);

    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("indirect trace"),
      bind_group_layouts: &groups,
      ..Default::default()
    });

    create_checked(device, "indirect trace", || {
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("indirect trace"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
          module: &module,
          entry_point: Some("vs_fullscreen"),
          compilation_options: Default::default(),
          buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
          module: &module,
          entry_point: Some("fs_indirect_trace"),
          compilation_options: Default::default(),
          targets: &[Some(VbaoPass::LIGHT.into()), Some(Self::HITS.into())],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: source.get_driver_cache(),
      })
    })
    .map(Some)
  }
}
