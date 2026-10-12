use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, FrameGraph, GraphBindings, GraphBufferAccess, PassParameters};

use crate::context::gpu_requirements::can_trace;
use crate::pass::reflection_scene_parameters::ReflectionSceneParameters;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::trace_request::TraceRequest;
use crate::pass::trace_request_handles::TraceRequestHandles;
use crate::pass::trace_request_list::TraceRequestList;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// The reflections' scene pass, the surfaces' and the water's alike: a pass listing the rays it leaves unanswered
/// (`TraceRequestList`) has its list emptied before it, then the scene pass traces exactly the rays listed against the
/// level's static geometry by hardware ray query (`frame/reflection_scene`), as many workgroups as were counted, and
/// writes each that meets something into the list's target. Nothing traces on a device without ray queries.
pub struct SceneTracePass {
  view_layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  /// The pass's own, the trace's and the texture cache's layouts, on a device with ray queries.
  layouts: Option<[wgpu::BindGroupLayout; 3]>,
  pipeline: Option<wgpu::ComputePipeline>,
  /// What a list's head is set back to before the pass filling it: `TraceRequest::HEAD`.
  head_reset: wgpu::Buffer,
  /// What a list keeping no ends binds for them, which nothing writes.
  no_ends: wgpu::TextureView,
  generation: u64,
}

impl SceneTracePass {
  /// Where the pass binds the texture cache's group, after `TraceParameters`.
  const TEXTURE_GROUP: u32 = 4;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layouts: Option<[wgpu::BindGroupLayout; 3]> = can_trace(device).then(|| {
      [
        ReflectionSceneParameters::create_layout(device),
        TraceParameters::create_layout(device),
        texture_layout.clone(),
      ]
    });

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, (view_layout, sky_layout), layouts.as_ref())?,
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
      generation: shaders.get_generation(),
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(
        device,
        shaders,
        (&self.view_layout, &self.sky_layout),
        self.layouts.as_ref(),
      ) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Scene trace rejected, tracing with the last one: {error}"),
      }
    }
  }

  /// Whether it traces rays: a device with ray queries.
  pub fn is_traced(&self) -> bool {
    self.layouts.is_some()
  }

  /// What a list keeping no ends binds for them.
  pub fn get_no_ends(&self) -> &wgpu::TextureView {
    &self.no_ends
  }

  /// Imports a list into the frame's graph and empties it for the pass that fills it, which binds what this answers.
  pub fn add_reset<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    (label, list): (&'static str, &'a TraceRequestList),
  ) -> TraceRequestHandles {
    let handles: TraceRequestHandles = TraceRequestHandles {
      requests: bindings.import_buffer(graph, label, &list.requests),
      head: bindings.import_buffer(graph, label, &list.head),
      dispatch: bindings.import_buffer(graph, label, &list.dispatch),
    };
    let reset = bindings.import_buffer(graph, "scene trace request head reset", &self.head_reset);
    let head = handles.head;

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

  /// Traces the rays a list holds once the pass filling it has run: its dispatch copied out of its head, then the
  /// scene pass over `parameters`, which name the list and the targets its hits write.
  pub fn add_trace<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    (names, handles): ((&'static str, &'static str), TraceRequestHandles),
    view: &'a ViewBinding,
    (parameters, sky): (ReflectionSceneParameters<'a>, SkyParameters<'a>),
    (trace, textures): (TraceParameters<'a>, &'a wgpu::BindGroup),
  ) {
    let (head, dispatch) = (handles.head, handles.dispatch);

    graph
      .add_encoder_pass(names.0)
      .buffer(head, GraphBufferAccess::CopySource)
      .buffer(dispatch, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let (head, copied) = (context.get_buffer(head), context.get_buffer(dispatch));

        context.get_encoder().copy_buffer_to_buffer(head, 4, copied, 0, 12);
      });
    graph
      .add_compute_pass(names.1)
      .parameters(&parameters)
      .parameters(&sky)
      .parameters(&trace)
      .buffer(dispatch, GraphBufferAccess::Indirect)
      .record(move |context| {
        let copied: &wgpu::Buffer = context.get_buffer(dispatch);

        self.record(context, view, (&parameters, &sky), (&trace, textures), copied);
      });
  }

  fn record(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    (parameters, sky): (&ReflectionSceneParameters<'_>, &SkyParameters<'_>),
    (trace, textures): (&TraceParameters<'_>, &wgpu::BindGroup),
    dispatch: &wgpu::Buffer,
  ) {
    let Some(pipeline) = &self.pipeline else {
      return;
    };

    context.bind(parameters);
    context.bind(sky);
    context.bind(trace);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(Self::TEXTURE_GROUP, textures, &[]);
    pass.dispatch_workgroups_indirect(dispatch, 0);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    (view_layout, sky_layout): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
    layouts: Option<&[wgpu::BindGroupLayout; 3]>,
  ) -> XrfResult<Option<wgpu::ComputePipeline>> {
    let Some([scene_layout, trace_layout, texture_layout]) = layouts else {
      return Ok(None);
    };
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/reflection_scene")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("scene trace"),
      bind_group_layouts: &[
        Some(view_layout),
        Some(scene_layout),
        Some(sky_layout),
        Some(trace_layout),
        Some(texture_layout),
      ],
      ..Default::default()
    });

    create_checked(device, "cs_scene", || {
      device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("scene trace"),
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("cs_scene"),
        compilation_options: Default::default(),
        cache: None,
      })
    })
    .map(Some)
  }
}
