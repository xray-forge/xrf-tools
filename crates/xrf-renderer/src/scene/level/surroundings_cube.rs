use glam::{Mat4, Vec3};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphRuntime,
  GraphTexture, GraphTextureDescriptor, StorageArray, StorageArrayMut, StorageValue, UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_reflection_probe::RenderReflectionProbe;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::view_targets::ViewTargets;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::shadow_cull::ShadowCull;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::surroundings_parameters::SurroundingsParameters;
use crate::pass::surroundings_pass::SurroundingsPass;
use crate::pass::surroundings_uniform::SurroundingsUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::static_layout::StaticLayout;

/// The level around the camera as a cube, a face a frame: the static scene culled and drawn into a small G-buffer by
/// a camera looking down one axis from where the viewport's stands, then lit into the prefiltered sky cube's first
/// mip, so a reflection that meets nothing on the screen reads the trees and the hills around it rather than the sky
/// alone. Camera-centred: a face at most a frame, each in turn, drawn again only where the camera moved from where it
/// was drawn, the sun turned, the scene's contents or the probe changed, or it has stood `REFRESH_FRAMES`, so a still
/// view draws a face every so often rather than every frame.
pub struct SurroundingsCube {
  lists: GrowableBuffer,
  args: wgpu::Buffer,
  view: ViewBinding,
  /// The face this frame draws, and the sun's direction for it.
  face: u32,
  to_sun: Vec3,
  /// Texels a side the cube's faces are, as the settings ask.
  size: u32,
  is_due: bool,
  /// Whether it stands in the prefiltered cube's first mip this frame, drawn or kept: none while it rests, when the
  /// prefilter fills that mip with the sky.
  is_active: bool,
  /// Where each face was last drawn from: the camera, the sun's direction, what the scene held, its size and the
  /// frame it was drawn on; none before it was.
  drawn: [Option<FaceDrawn>; 6],
  /// Frames prepared since it was made.
  frame: u32,
}

/// Where a face was drawn from.
#[derive(Clone, Copy)]
struct FaceDrawn {
  position: Vec3,
  to_sun: Vec3,
  contents: usize,
  size: u32,
  frame: u32,
}

impl SurroundingsCube {
  /// Texels a side each face's G-buffer is drawn at over the cube's: twice, averaged down, so a thin tree does not
  /// alias out of the probe.
  pub const DRAWN_OVER: u32 = 2;
  /// Metres the faces see from the camera: from past what stands about it, which is the screen's to trace and, drawn
  /// into a cube, showed in every far-ended reflection wherever the camera stood, to the whole of a level.
  const NEAR: f32 = 15.0;
  const FAR: f32 = 1200.0;
  /// Metres the camera moves, and the cosine of the angle the sun turns, before a face is drawn again; and the frames
  /// after which it is drawn again whatever moved, so the weather's light comes through.
  const MOVED: f32 = 1.0;
  const SUN_TURNED: f32 = 0.9994;
  const REFRESH_FRAMES: u32 = 30;

  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64) -> Self {
    Self {
      lists: GrowableBuffer::new(device, "surroundings", wgpu::BufferUsages::STORAGE),
      args: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("surroundings"),
        size: args_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      view: ViewBinding::new(device, view_layout),
      face: 5,
      to_sun: Vec3::Y,
      size: RenderReflectionProbe::Fine.get_size(),
      is_due: false,
      is_active: false,
      drawn: [None; 6],
      frame: 0,
    }
  }

  /// Whether this frame draws a face, as `prepare` decided.
  pub fn is_due(&self) -> bool {
    self.is_due
  }

  /// Whether it stands in the prefiltered cube's first mip this frame, a face drawn or all kept as they were.
  pub fn is_active(&self) -> bool {
    self.is_active
  }

  /// Draws no face this frame: the reflections are not traced, so nothing reads the cube. Its faces are forgotten,
  /// since the prefilter fills their mip with the sky meanwhile.
  pub fn rest(&mut self) {
    self.is_due = false;
    self.is_active = false;
    self.drawn = [None; 6];
  }

  /// Readies the next face's camera from where the viewport's stands, looking down its axis with the up a cube
  /// texture's face lays its texels by; `encoder` takes the copies a list growing makes.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame: &ShadowFrame<'_>,
    probe: RenderReflectionProbe,
  ) {
    let face: u32 = (self.face + 1) % 6;
    let drawn: u32 = probe.get_size() * Self::DRAWN_OVER;
    let position: Vec3 = frame.camera.position;
    let to_sun: Vec3 = -frame.sun_direction;
    let contents: usize = frame.scene.get_contents();

    self.frame = self.frame.wrapping_add(1);
    self.face = face;
    self.is_active = true;

    let is_stale: bool = self.drawn[face as usize].is_none_or(|it: FaceDrawn| {
      it.position.distance(position) > Self::MOVED
        || it.to_sun.dot(to_sun) < Self::SUN_TURNED
        || it.contents != contents
        || it.size != probe.get_size()
        || self.frame.wrapping_sub(it.frame) >= Self::REFRESH_FRAMES
    });

    if !is_stale {
      self.is_due = false;

      return;
    }

    let (forward, up): (Vec3, Vec3) = match face {
      0 => (Vec3::X, Vec3::Y),
      1 => (Vec3::NEG_X, Vec3::Y),
      2 => (Vec3::Y, Vec3::NEG_Z),
      3 => (Vec3::NEG_Y, Vec3::Z),
      4 => (Vec3::Z, Vec3::Y),
      _ => (Vec3::NEG_Z, Vec3::Y),
    };
    let look: Mat4 = glam::camera::rh::view::look_at_mat4(position, position + forward, up);
    let view: CameraView = CameraView::new(position, look, 90.0, 1.0, Self::NEAR, Self::FAR);
    let scene = frame.scene;

    self
      .lists
      .reserve(device, encoder, (scene.get_list_capacity().max(1) as u64) * 8);
    queue.write_buffer(&self.args, 0, bytemuck::cast_slice(scene.get_initial_args()));
    self.view.write(
      queue,
      &CameraUniform::new(
        &view,
        RenderRect {
          x: 0,
          y: 0,
          width: drawn,
          height: drawn,
        },
        frame.switches,
      ),
    );

    self.to_sun = to_sun;
    self.size = probe.get_size();
    self.is_due = true;
    self.drawn[face as usize] = Some(FaceDrawn {
      position,
      to_sun,
      contents,
      size: self.size,
      frame: self.frame,
    });
  }

  /// Declares the face's cull, its draw into a G-buffer of its own and its resolve into `faces`, the prefiltered
  /// cube's first mip, where this frame's `prepare` readied one.
  #[allow(clippy::too_many_arguments)]
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    scene: &StaticSceneHandles,
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
    (lit, sky): (&LightingHandles<'a>, &SkyParameters<'a>),
    faces: GraphTexture,
  ) {
    if !self.is_due {
      return;
    }

    let args: GraphBuffer = bindings.import_buffer(graph, SurroundingsPass::DRAW_PASS, &self.args);
    let lists: GraphBuffer = bindings.import_buffer(graph, SurroundingsPass::CULL_PASS, self.lists.get_buffer());
    let cull: StaticCullParameters = scene.get_cull_parameters(StorageArrayMut::new(lists), StorageArrayMut::new(args));
    let layouts: [StaticDrawParameters; StaticLayout::COUNT] = scene.get_layout_draws(StorageArray::new(lists));
    let draws: StaticDraws = StaticDraws {
      layouts,
      impostors: scene.get_camera_draws().impostors,
    };
    let size: u32 = self.size;
    let drawn: u32 = size * Self::DRAWN_OVER;
    let gbuffer: [GraphTexture; 4] = [
      ("surroundings albedo", ViewTargets::ALBEDO),
      ("surroundings normal", ViewTargets::NORMAL),
      ("surroundings material", ViewTargets::MATERIAL),
      ("surroundings motion", ViewTargets::MOTION),
    ]
    .map(|(label, format)| graph.create_texture(GraphTextureDescriptor::new_2d(label, drawn, drawn, format)));
    let depth: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "surroundings depth",
      drawn,
      drawn,
      ViewTargets::DEPTH,
    ));

    // The shadow's cull: every tree row as a mesh whatever its level of detail, no water, nothing hidden early.
    graph
      .add_compute_pass(SurroundingsPass::CULL_PASS)
      .parameters(&cull)
      .record(move |context| {
        passes
          .cull
          .record_shadow(context, &self.view, &cull, params, ShadowCull::Cascade);
      });

    let builder = gbuffer
      .iter()
      .fold(
        graph.add_raster_pass(SurroundingsPass::DRAW_PASS),
        |builder, texture| {
          builder.color(GraphColorAttachment::new(
            *texture,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
        },
      )
      .depth(GraphDepthAttachment::new(depth, wgpu::LoadOp::Clear(0.0)))
      .buffer(args, GraphBufferAccess::Indirect);

    StaticDraws::declare_layouts(builder, &layouts).record(move |context| {
      let args: &wgpu::Buffer = context.get_buffer(args);

      passes
        .gbuffer
        .record(context, (&self.view, &draws, textures), args, false);
    });

    let settings: UniformBinding<SurroundingsUniform> =
      runtime.push_uniform(&SurroundingsUniform::new(self.to_sun, self.face, (size, drawn)));
    let parameters: SurroundingsParameters = SurroundingsParameters {
      depth_target: depth,
      albedo_target: gbuffer[0],
      normal_target: gbuffer[1],
      material_target: gbuffer[2],
      material_lut: lit.material_lut,
      lut_sampler: lit.lut_sampler,
      sky_cube_0: sky.sky_cube_0,
      sky_cube_1: sky.sky_cube_1,
      sky_environment_0: sky.sky_environment_0,
      sky_environment_1: sky.sky_environment_1,
      sky_clouds_0: sky.sky_clouds_0,
      sky_clouds_1: sky.sky_clouds_1,
      sky_clamp: sky.sky_clamp,
      sky_repeat: sky.sky_repeat,
      lighting: lit.lighting,
      exposure: StorageValue::new(lit.exposure),
      settings,
      faces,
    };

    graph
      .add_compute_pass(SurroundingsPass::RESOLVE_PASS)
      .parameters(&parameters)
      .record(move |context| {
        passes.surroundings.record(context, &self.view, size, &parameters);
      });
  }
}
