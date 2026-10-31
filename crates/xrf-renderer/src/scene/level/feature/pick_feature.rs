use xrf_error::XrfResult;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphTexture,
  GraphTextureAccess, RenderPipelineDescription, VertexLayout,
};

use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::pick_target::PickTarget;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::helpers_feature::HelperDraws;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::helpers::helper_icon_instance::HelperIconInstance;
use crate::scene::level::helpers::helper_vertex::HelperVertex;

/// What a view's picks are drawn into and read back from: the one-texel target with its readbacks, and the narrowed
/// camera the frame's visible clusters and its helpers' wires and icons are drawn again through, both made on the first
/// pick.
#[derive(Default)]
pub struct PickFeature {
  target: Option<PickTarget>,
  view: Option<ViewBinding>,
  /// The layout the narrowed camera binds by, which the wires' and icons' pipelines are made with.
  view_layout: Option<wgpu::BindGroupLayout>,
}

impl PickFeature {
  /// Readies a pick's `camera` and target, and answers the readback it will be copied into; none while every readback
  /// is on its way.
  pub fn ready(
    &mut self,
    (device, encoder): (&wgpu::Device, &mut FrameEncoder),
    camera: &CameraUniform,
    view_layout: &wgpu::BindGroupLayout,
  ) -> Option<usize> {
    let slot: usize = self
      .target
      .get_or_insert_with(|| PickTarget::new(device))
      .find_free_readback()?;

    self.view_layout.get_or_insert_with(|| view_layout.clone());
    self
      .view
      .get_or_insert_with(|| ViewBinding::new(device, view_layout))
      .write(encoder, camera);

    Some(slot)
  }

  /// Adds a pick into its readback `slot`: the frame's visible clusters drawn again into its texel, through its
  /// narrowed camera, after every cull of the frame, passing the composited surfaces by where `is_through`, then the
  /// helpers' wires wide and icons whole over them; then the texel copied out into its readback.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    (gbuffer, pipelines): (&'a StaticGBufferPass, PipelineSource<'_>),
    (textures, scene, helpers): (&'a wgpu::BindGroup, &StaticSceneHandles, HelperDraws<'a>),
    (slot, is_through): (usize, bool),
  ) {
    let (Some(target), Some(view)) = (&self.target, &self.view) else {
      return;
    };
    let color: GraphTexture = bindings.import_view(graph, "pick", &target.color);
    let depth: GraphTexture = bindings.import_view(graph, "pick depth", &target.depth);
    let draws: StaticDraws = scene.get_camera_draws();
    let args: [GraphBuffer; 2] = [scene.args, scene.late];
    // A pipeline for what of it is drawn, wires and icons each; a draw is one instance a wire or an icon.
    let create = |(vertex, layout): (&'static str, VertexLayout),
                  parts: [Option<(&'a wgpu::Buffer, u32)>; 2],
                  per_instance: u32|
     -> Option<(Vec<(&'a wgpu::Buffer, u32)>, wgpu::RenderPipeline)> {
      let draws: Vec<(&wgpu::Buffer, u32)> = parts
        .into_iter()
        .flatten()
        .map(|(buffer, count)| (buffer, count / per_instance))
        .collect();

      if draws.is_empty() {
        return None;
      }

      self
        .create_helper_pipeline(pipelines, (vertex, "fs_helper_pick"), layout)
        .inspect_err(|error| log::error!("Helper pick shader rejected, picking no helper: {error}"))
        .ok()
        .map(|pipeline| (draws, pipeline))
    };
    let picked = [
      create(
        (
          "vs_helper_pick",
          VertexLayout::new(
            HelperVertex::SIZE * 2,
            wgpu::VertexStepMode::Instance,
            &HelperVertex::SEGMENT_ATTRIBUTES,
          ),
        ),
        helpers.wires,
        2,
      ),
      create(
        (
          "vs_helper_icon_pick",
          VertexLayout::new(
            HelperIconInstance::SIZE,
            wgpu::VertexStepMode::Instance,
            &HelperIconInstance::PICK_ATTRIBUTES,
          ),
        ),
        helpers.icons,
        1,
      ),
    ];

    draws
      .declare(
        graph
          .add_raster_pass("static pick")
          .color(GraphColorAttachment::new(
            color,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .depth(GraphDepthAttachment::new(depth, wgpu::LoadOp::Clear(0.0)))
          .buffer(args[0], GraphBufferAccess::Indirect)
          .buffer(args[1], GraphBufferAccess::Indirect),
        true,
      )
      .record(move |context| {
        let args: [&wgpu::Buffer; 2] = args.map(|args| context.get_buffer(args));

        gbuffer.pick(context, (view, &draws, textures), &args, is_through);

        let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

        for (draws, pipeline) in picked.iter().flatten() {
          pass.set_pipeline(pipeline);
          pass.set_bind_group(0, &view.bind_group, &[]);

          for (buffer, instances) in draws {
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..6, 0..*instances);
          }
        }
      });

    graph
      .add_encoder_pass("pick copy")
      .texture(color, GraphTextureAccess::CopySource)
      .keep()
      .record(move |context| target.copy_out(context.get_encoder(), slot));
  }

  /// A helper pick pipeline: each wire or icon an instance read by `layout`, drawn over whatever the clusters left,
  /// with no depth test.
  fn create_helper_pipeline(
    &self,
    pipelines: PipelineSource<'_>,
    (vertex, fragment): (&'static str, &'static str),
    layout: VertexLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    pipelines.get_render_pipeline(
      &RenderPipelineDescription::new("helper pick", "frame/helper_pick", vertex)
        .with_fragment(fragment)
        .with_bind_group_layouts(&[self.view_layout.as_ref()])
        .with_vertex_layout(layout)
        .with_depth_stencil(wgpu::DepthStencilState {
          format: PickTarget::DEPTH_FORMAT,
          depth_write_enabled: Some(false),
          depth_compare: Some(wgpu::CompareFunction::Always),
          stencil: wgpu::StencilState::default(),
          bias: wgpu::DepthBiasState::default(),
        })
        .with_target(PickTarget::FORMAT.into()),
    )
  }

  /// Asks for a pick's texel back, its frame just submitted.
  pub fn request(&self, slot: usize) {
    if let Some(target) = &self.target {
      target.request(slot);
    }
  }

  /// A pick's texel once it is back; a view made since the pick has no readback for it, and it met nothing still drawn.
  pub fn take(&self, slot: usize) -> Option<XrfResult<[u32; 4]>> {
    match &self.target {
      Some(target) => target.take(slot),
      None => Some(Ok([0; 4])),
    }
  }

  /// Drops the pick a frame that was never submitted copied out.
  pub fn abandon(&self) {
    if let Some(target) = &self.target {
      target.abandon();
    }
  }
}

impl ViewFeature for PickFeature {
  /// Nothing counted: one texel and its readbacks are all it holds.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {
    self.target = None;
    self.view = None;
    self.view_layout = None;
  }
}
