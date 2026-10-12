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
use crate::scene::level::frame_load::FrameLoad;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::surroundings_faces::SurroundingsFaces;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::static_layout::StaticLayout;

/// The level around the camera as a cube, a face a frame: the static scene culled and drawn into a G-buffer of the
/// face's own by a camera looking down one axis from where the viewport's stands, then lit into the prefiltered sky
/// cube's first mip, so a reflection that meets nothing on the screen reads the trees and the hills around it rather
/// than the sky alone. A face is drawn again only where the camera moved from where it was drawn or the scene's
/// contents changed; where the sun turned or it has stood `REFRESH_FRAMES`, it is only lit again from the G-buffer it
/// keeps, which costs next to nothing, so a still view draws no geometry at all. A face never drawn at the probe's size
/// draws at once; any other waits for a frame light on other work (`FrameLoad`), so its cost never lands on a frame
/// already heavy with the sun's far cascades.
pub struct SurroundingsCube {
  lists: GrowableBuffer,
  args: wgpu::Buffer,
  view: ViewBinding,
  /// The faces' G-buffers, while the cube is in use.
  faces: Option<SurroundingsFaces>,
  /// The face this frame lights, or last lit, and the sun's direction for it.
  face: u32,
  to_sun: Vec3,
  /// Texels a side the cube's faces are, as the settings ask.
  size: u32,
  /// Whether this frame lights a face, and whether it draws its geometry first.
  is_due: bool,
  is_redrawn: bool,
  /// Whether it stands in the prefiltered cube's first mip this frame, drawn or kept: none while it rests, when the
  /// prefilter fills that mip with the sky.
  is_active: bool,
  /// Where each face was last drawn from and when it was last lit; none before it was drawn.
  drawn: [Option<FaceDrawn>; 6],
  /// Frames prepared since it was made.
  frame: u32,
  /// Frames a face has waited, due, for a frame light on other work.
  waited: u32,
}

/// Where a face was drawn from: the camera, what the scene held and its size; and the sun's direction and the frame it
/// was last lit with.
#[derive(Clone, Copy)]
struct FaceDrawn {
  position: Vec3,
  contents: usize,
  size: u32,
  to_sun: Vec3,
  lit: u32,
}

impl SurroundingsCube {
  /// Texels a side each face's G-buffer is drawn at over the cube's: twice, averaged down, so a thin tree does not
  /// alias out of the probe; never past `DRAWN_MOST`, which bounds what the six G-buffers keep.
  pub const DRAWN_OVER: u32 = 2;
  pub const DRAWN_MOST: u32 = 1024;
  /// Metres the faces see from the camera: from past what stands about it, which is the screen's to trace and, drawn
  /// into a cube, showed in every far-ended reflection wherever the camera stood, to the whole of a level.
  const NEAR: f32 = 15.0;
  const FAR: f32 = 1200.0;
  /// Metres the camera moves before a face is drawn again; the cosine of the angle the sun turns before it is lit
  /// again, and the frames after which it is lit again whatever turned, so the weather's light comes through.
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
      faces: None,
      face: 5,
      to_sun: Vec3::Y,
      size: RenderReflectionProbe::Fine.get_size(),
      is_due: false,
      is_redrawn: false,
      is_active: false,
      drawn: [None; 6],
      frame: 0,
      waited: 0,
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

  /// Texels a side a face is drawn at for a cube `size` texels a side.
  pub fn get_drawn_size(size: u32) -> u32 {
    (size * Self::DRAWN_OVER).min(Self::DRAWN_MOST).max(size)
  }

  /// Draws no face this frame: nothing reads the cube. Its faces and their G-buffers go, since the prefilter fills
  /// their mip with the sky meanwhile.
  pub fn rest(&mut self) {
    self.is_due = false;
    self.is_redrawn = false;
    self.is_active = false;
    self.drawn = [None; 6];
    self.faces = None;
  }

  /// Readies the camera of the face due, if one is, looking down its axis with the up a cube texture's face lays its
  /// texels by: one never drawn at this size first, then one the camera or the scene left behind, each after the last
  /// in turn, both drawn from where the viewport's camera stands; else one the sun turned from, then the one lit
  /// longest ago past `REFRESH_FRAMES`, both lit again from where they were drawn. `encoder` takes the copies a list
  /// growing makes.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame: &ShadowFrame<'_>,
    (probe, load): (RenderReflectionProbe, &mut FrameLoad),
  ) {
    let size: u32 = probe.get_size();
    let drawn: u32 = Self::get_drawn_size(size);
    let camera: Vec3 = frame.camera.position;
    let to_sun: Vec3 = -frame.sun_direction;
    let contents: usize = frame.scene.get_contents();

    self.frame = self.frame.wrapping_add(1);
    self.is_active = true;
    self.is_due = false;
    self.is_redrawn = false;

    if self.faces.as_ref().is_none_or(|faces| faces.drawn != drawn) {
      self.faces = Some(SurroundingsFaces::new(device, drawn));
      self.drawn = [None; 6];
    }

    let order = || (1..=6).map(|step| (self.face + step) % 6);
    let unready: Option<u32> = order().find(|face| self.drawn[*face as usize].is_none_or(|it| it.size != size));
    let (face, is_redrawn): (u32, bool) = match unready {
      Some(face) => (face, true),
      None => {
        let moved: Option<u32> = order().find(|face| {
          self.drawn[*face as usize]
            .is_some_and(|it: FaceDrawn| it.position.distance(camera) > Self::MOVED || it.contents != contents)
        });

        match moved {
          Some(face) => {
            if !load.take(self.waited) {
              self.waited += 1;

              return;
            }

            (face, true)
          }
          None => {
            let age = |face: u32| self.drawn[face as usize].map_or(0, |it| self.frame.wrapping_sub(it.lit));
            let turned: Option<u32> = order().find(|face| {
              self.drawn[*face as usize].is_some_and(|it: FaceDrawn| it.to_sun.dot(to_sun) < Self::SUN_TURNED)
            });
            let aged: Option<u32> = (0..6)
              .filter(|face| age(*face) >= Self::REFRESH_FRAMES)
              .max_by_key(|face| age(*face));
            let Some(face) = turned.or(aged) else {
              self.waited = 0;

              return;
            };

            (face, false)
          }
        }
      }
    };
    // A face drawn again is seen from the camera; one lit again, from where its G-buffer was drawn.
    let position: Vec3 = match self.drawn[face as usize] {
      Some(it) if !is_redrawn => it.position,
      _ => camera,
    };

    self.face = face;
    self.waited = 0;

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

    if is_redrawn {
      self
        .lists
        .reserve(device, encoder, (scene.get_list_capacity().max(1) as u64) * 8);
      queue.write_buffer(&self.args, 0, bytemuck::cast_slice(scene.get_initial_args()));
    }

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
    self.size = size;
    self.is_due = true;
    self.is_redrawn = is_redrawn;
    self.drawn[face as usize] = Some(FaceDrawn {
      position,
      contents: match self.drawn[face as usize] {
        Some(it) if !is_redrawn => it.contents,
        _ => contents,
      },
      size,
      to_sun,
      lit: self.frame,
    });
  }

  /// Declares the face's cull and its draw into the G-buffer it keeps where it is drawn again, then its resolve into
  /// `faces`, the prefiltered cube's first mip, where this frame's `prepare` readied one.
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
    let (true, Some(kept)) = (self.is_due, &self.faces) else {
      return;
    };
    let size: u32 = self.size;
    let drawn: u32 = kept.drawn;
    let [albedo, normal, material, depth]: [GraphTexture; 4] = std::array::from_fn(|target| {
      bindings.import_view(
        &mut *graph,
        SurroundingsFaces::FORMATS[target].0,
        &kept.layers[self.face as usize][target],
      )
    });

    if self.is_redrawn {
      self.add_draw_passes(
        (graph, bindings),
        passes,
        scene,
        (params, textures),
        [albedo, normal, material, depth],
      );
    }

    let settings: UniformBinding<SurroundingsUniform> =
      runtime.push_uniform(&SurroundingsUniform::new(self.to_sun, self.face, (size, drawn)));
    let parameters: SurroundingsParameters = SurroundingsParameters {
      depth_target: depth,
      albedo_target: albedo,
      normal_target: normal,
      material_target: material,
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

  /// Declares the face's cull and its draw into its G-buffer, `targets` its albedo, normal, material and depth.
  fn add_draw_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    scene: &StaticSceneHandles,
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
    [albedo, normal, material, depth]: [GraphTexture; 4],
  ) {
    let args: GraphBuffer = bindings.import_buffer(graph, SurroundingsPass::DRAW_PASS, &self.args);
    let lists: GraphBuffer = bindings.import_buffer(graph, SurroundingsPass::CULL_PASS, self.lists.get_buffer());
    let cull: StaticCullParameters = scene.get_cull_parameters(StorageArrayMut::new(lists), StorageArrayMut::new(args));
    let layouts: [StaticDrawParameters; StaticLayout::COUNT] = scene.get_layout_draws(StorageArray::new(lists));
    let draws: StaticDraws = StaticDraws {
      layouts,
      impostors: scene.get_camera_draws().impostors,
    };
    let drawn: u32 = self.faces.as_ref().map_or(0, |faces| faces.drawn);
    // The G-buffer pass writes a motion target too, which nothing reads of a face.
    let motion: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "surroundings motion",
      drawn,
      drawn,
      ViewTargets::MOTION,
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

    let builder = [albedo, normal, material, motion]
      .into_iter()
      .fold(
        graph.add_raster_pass(SurroundingsPass::DRAW_PASS),
        |builder, texture| {
          builder.color(GraphColorAttachment::new(
            texture,
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
  }
}
