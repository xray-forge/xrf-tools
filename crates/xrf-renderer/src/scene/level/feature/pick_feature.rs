use xrf_error::XrfResult;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphTexture,
  GraphTextureAccess,
};

use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::pick_target::PickTarget;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;

/// What a view's picks are drawn into and read back from: the one-texel target with its readbacks, and the narrowed
/// camera the frame's visible clusters are drawn again through, both made on the first pick.
#[derive(Default)]
pub struct PickFeature {
  target: Option<PickTarget>,
  view: Option<ViewBinding>,
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

    self
      .view
      .get_or_insert_with(|| ViewBinding::new(device, view_layout))
      .write(encoder, camera);

    Some(slot)
  }

  /// Adds a pick into its readback `slot`: the frame's visible clusters drawn again into its texel, through its narrowed
  /// camera, after every cull of the frame, passing the composited surfaces by where `is_through`; then the texel
  /// copied out into its readback.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    gbuffer: &'a StaticGBufferPass,
    (textures, scene): (&'a wgpu::BindGroup, &StaticSceneHandles),
    (slot, is_through): (usize, bool),
  ) {
    let (Some(target), Some(view)) = (&self.target, &self.view) else {
      return;
    };
    let color: GraphTexture = bindings.import_view(graph, "pick", &target.color);
    let depth: GraphTexture = bindings.import_view(graph, "pick depth", &target.depth);
    let draws: StaticDraws = scene.get_camera_draws();
    let args: [GraphBuffer; 2] = [scene.args, scene.late];

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
      });

    graph
      .add_encoder_pass("pick copy")
      .texture(color, GraphTextureAccess::CopySource)
      .keep()
      .record(move |context| target.copy_out(context.get_encoder(), slot));
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
  /// One texel and its readbacks.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {
    self.target = None;
    self.view = None;
  }
}
