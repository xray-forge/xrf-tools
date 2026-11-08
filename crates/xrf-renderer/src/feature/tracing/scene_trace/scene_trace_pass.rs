use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{
  ComputeContext, ComputePipelineDescription, FrameGraph, GraphBindings, GraphBufferAccess, PassParameters, PipelineSet,
};

use crate::context::gpu_requirements::can_trace;
use crate::feature::tracing::scene_trace::reflection_scene_parameters::ReflectionSceneParameters;
use crate::feature::tracing::scene_trace::requests::scene_list_parameters::SceneListParameters;
use crate::feature::tracing::scene_trace::requests::trace_request::TraceRequest;
use crate::feature::tracing::scene_trace::requests::trace_request_handles::TraceRequestHandles;
use crate::feature::tracing::scene_trace::requests::trace_request_list::TraceRequestList;
use crate::feature::tracing::scene_trace::requests::water_ray_parameters::WaterRayParameters;
use crate::feature::tracing::scene_trace::requests::water_rays::WaterRays;
use crate::feature::tracing::scene_trace::scene_trace_bindings::SceneTraceBindings;
use crate::feature::tracing::scene_trace::scene_trace_request::SceneTraceRequest;
use crate::feature::tracing::view_trace::trace_parameters::TraceParameters;
use crate::pass::build::pipeline_source::PipelineSource;

/// The scene pass's two ways in: the rays a list holds, a thread a request, and the water's every-pixel rays
/// (`WaterRays`), a thread a drawn pixel.
struct SceneTracePipelines {
  listed: wgpu::ComputePipeline,
  pixels: wgpu::ComputePipeline,
}

/// The layouts the scene pass's pipelines bind beside the view's and the sky's, on a device with ray queries: its own
/// group, the trace's, the texture cache's, and each way in's rays.
struct SceneTraceLayouts {
  scene: wgpu::BindGroupLayout,
  trace: wgpu::BindGroupLayout,
  textures: wgpu::BindGroupLayout,
  list: wgpu::BindGroupLayout,
  water_rays: wgpu::BindGroupLayout,
}

/// The reflections' scene pass, the surfaces' and the water's alike: a pass listing the rays it leaves unanswered
/// (`TraceRequestList`) has its list emptied before it, then the scene pass traces exactly the rays listed against the
/// level's static geometry by hardware ray query (`frame/reflection_scene`), as many workgroups as were counted, and
/// writes each that meets something into the list's target; the water tracing every pixel leaves a record at each
/// pixel instead (`WaterRays`), which the pass traces a thread a pixel. Nothing traces on a device without ray queries.
pub(crate) struct SceneTracePass {
  view_layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  layouts: Option<SceneTraceLayouts>,
  pipelines: PipelineSet<Option<SceneTracePipelines>>,
  /// What a list's head is set back to before the pass filling it, its wait kept: `TraceRequest::HEAD`.
  head_reset: wgpu::Buffer,
  /// What a list keeping no ends binds for them, which nothing writes.
  no_ends: wgpu::TextureView,
}

impl SceneTracePass {
  /// Where the pass binds the texture cache's group, after `TraceParameters`.
  const TEXTURE_GROUP: u32 = 4;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub(crate) fn new(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layouts: Option<SceneTraceLayouts> = can_trace(device).then(|| SceneTraceLayouts {
      scene: ReflectionSceneParameters::create_layout(device),
      trace: TraceParameters::create_layout(device),
      textures: texture_layout.clone(),
      list: SceneListParameters::create_layout(device),
      water_rays: WaterRayParameters::create_layout(device),
    });

    Ok(Self {
      pipelines: PipelineSet::new(
        source.cache,
        Self::create_pipelines(source, view_layout, sky_layout, layouts.as_ref())?,
      ),
      view_layout: view_layout.clone(),
      sky_layout: sky_layout.clone(),
      layouts,
      head_reset: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("scene trace request head reset"),
        contents: bytemuck::cast_slice(&TraceRequest::HEAD),
        usage: wgpu::BufferUsages::COPY_SRC,
      }),
      no_ends: device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some("scene trace no ends"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: wgpu::TextureFormat::R32Float,
          usage: wgpu::TextureUsages::STORAGE_BINDING,
          view_formats: &[],
        })
        .create_view(&Default::default()),
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub(crate) fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self.pipelines.refresh(source.cache, || {
      Self::create_pipelines(source, &self.view_layout, &self.sky_layout, self.layouts.as_ref())
    }) {
      log::error!("Scene trace rejected, tracing with the last one: {error}");
    }
  }

  /// Whether it traces rays: a device with ray queries.
  pub(crate) fn is_traced(&self) -> bool {
    self.layouts.is_some()
  }

  /// What a list keeping no ends binds for them.
  pub(crate) fn get_no_ends(&self) -> &wgpu::TextureView {
    &self.no_ends
  }

  /// Imports a list into the frame's graph and empties it for the pass that fills it, which binds what this answers.
  pub(crate) fn add_reset<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    label: &'static str,
    list: &'a TraceRequestList,
  ) -> TraceRequestHandles {
    let handles: TraceRequestHandles = TraceRequestHandles {
      requests: bindings.import_buffer(graph, label, &list.requests),
      head: bindings.import_buffer(graph, label, &list.head),
      dispatch: bindings.import_buffer(graph, label, &list.dispatch),
      wait: bindings.import_buffer(graph, label, &list.wait),
    };
    let reset = bindings.import_buffer(graph, "scene trace request head reset", &self.head_reset);
    let (head, wait) = (handles.head, handles.wait);

    // The wait the last frame's scene pass left, read plainly by this frame's listing pass.
    graph
      .add_encoder_pass("scene trace request wait")
      .buffer(head, GraphBufferAccess::CopySource)
      .buffer(wait, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let (head, wait) = (context.get_buffer(head), context.get_buffer(wait));

        context.get_encoder().copy_buffer_to_buffer(
          head,
          TraceRequest::HEAD_WAIT_OFFSET,
          wait,
          0,
          size_of::<u32>() as u64,
        );
      });

    graph
      .add_encoder_pass(label)
      .buffer(reset, GraphBufferAccess::CopySource)
      .buffer(head, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let (reset, head) = (context.get_buffer(reset), context.get_buffer(head));

        context
          .get_encoder()
          .copy_buffer_to_buffer(reset, 0, head, 0, size_of_val(&TraceRequest::HEAD) as u64);
      });

    handles
  }

  /// Traces the rays a `request`'s list holds once the pass filling it has run: its dispatch copied out of its head,
  /// then the scene pass over the list and the `bindings`, whose parameters name the targets its hits write.
  pub(crate) fn add_trace<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    request: SceneTraceRequest,
    bindings: SceneTraceBindings<'a>,
  ) {
    let (head, dispatch) = (request.handles.head, request.handles.dispatch);
    let list: SceneListParameters = request.handles.to_read();

    graph
      .add_encoder_pass(request.dispatch_label)
      .buffer(head, GraphBufferAccess::CopySource)
      .buffer(dispatch, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let (head, copied) = (context.get_buffer(head), context.get_buffer(dispatch));

        context.get_encoder().copy_buffer_to_buffer(
          head,
          TraceRequest::HEAD_DISPATCH_OFFSET,
          copied,
          0,
          TraceRequest::DISPATCH_BYTES,
        );
      });
    graph
      .add_compute_pass(request.trace_label)
      .parameters(&bindings.parameters)
      .parameters(&bindings.sky)
      .parameters(&bindings.trace)
      .parameters(&list)
      .buffer(dispatch, GraphBufferAccess::Indirect)
      .record(move |context| {
        let copied: &wgpu::Buffer = context.get_buffer(dispatch);

        if let Some(pass) = self.begin(context, &bindings, &list, |pipelines| &pipelines.listed) {
          pass.dispatch_workgroups_indirect(copied, 0);
        }
      });
  }

  /// Traces the water's every-pixel rays (`rays`) once the pass writing them has run, as `label`: a thread a pixel of
  /// the drawn `size`, over the `bindings`, a record another frame wrote passed over.
  pub(crate) fn add_trace_pixels<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    label: &'static str,
    (width, height): (u32, u32),
    bindings: SceneTraceBindings<'a>,
    rays: WaterRayParameters,
  ) {
    graph
      .add_compute_pass(label)
      .parameters(&bindings.parameters)
      .parameters(&bindings.sky)
      .parameters(&bindings.trace)
      .parameters(&rays)
      .record(move |context| {
        if let Some(pass) = self.begin(context, &bindings, &rays, |pipelines| &pipelines.pixels) {
          pass.dispatch_workgroups(
            width.div_ceil(WaterRays::WORKGROUP),
            height.div_ceil(WaterRays::WORKGROUP),
            1,
          );
        }
      });
  }

  /// Binds the `bindings`, the way the rays come in (`rays`, at its own group) and the pipeline `pick` chooses, on a
  /// device that traces.
  fn begin<'c, P: PassParameters>(
    &self,
    context: &'c mut ComputeContext<'_>,
    bindings: &SceneTraceBindings<'_>,
    rays: &P,
    pick: impl Fn(&SceneTracePipelines) -> &wgpu::ComputePipeline,
  ) -> Option<&'c mut wgpu::ComputePass<'static>> {
    let Some(pipelines) = &*self.pipelines else {
      return None;
    };

    context.bind(&bindings.parameters);
    context.bind(&bindings.sky);
    context.bind(&bindings.trace);
    context.bind(rays);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(pick(pipelines));
    pass.set_bind_group(0, &bindings.view.bind_group, &[]);
    pass.set_bind_group(Self::TEXTURE_GROUP, bindings.textures, &[]);

    Some(pass)
  }

  /// Each way in binds its own rays' group, the list's (`SceneListParameters`) or the water's every-pixel records'
  /// (`WaterRayParameters`), and leaves the other's slot a gap.
  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
    layouts: Option<&SceneTraceLayouts>,
  ) -> XrfResult<Option<SceneTracePipelines>> {
    let Some(layouts) = layouts else {
      return Ok(None);
    };
    let pipeline = |label: &'static str,
                    entry: &'static str,
                    rays: [Option<&wgpu::BindGroupLayout>; 2]|
     -> XrfResult<wgpu::ComputePipeline> {
      source.get_compute_pipeline(
        &ComputePipelineDescription::new(label, "frame/reflection_scene", entry).with_bind_group_layouts(&[
          Some(view_layout),
          Some(&layouts.scene),
          Some(sky_layout),
          Some(&layouts.trace),
          Some(&layouts.textures),
          rays[0],
          rays[1],
        ]),
      )
    };

    Ok(Some(SceneTracePipelines {
      listed: pipeline("scene trace", "cs_scene", [Some(&layouts.list), None])?,
      pixels: pipeline(
        "scene trace pixels",
        "cs_water_scene",
        [None, Some(&layouts.water_rays)],
      )?,
    }))
  }
}
