use glam::{Mat4, Vec2, Vec3};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphRuntime, GraphTexture,
  GraphTextureDescriptor, StorageArray, StorageArrayMut, StorageValue, UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_reflection_probe::RenderReflectionProbe;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
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
use crate::pass::static_shadow_pass::StaticShadowPass;
use crate::pass::surroundings_parameters::SurroundingsParameters;
use crate::pass::surroundings_pass::SurroundingsPass;
use crate::pass::surroundings_uniform::SurroundingsUniform;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::shadow_cut::ShadowCut;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::surroundings_faces::SurroundingsFaces;
use crate::scene::static_scene::static_layout::StaticLayout;

/// The level around the camera as a cube, a face a frame: the static scene culled and drawn into a G-buffer of the
/// face's own by a camera looking down one axis from where the viewport's stands, then lit into the prefiltered sky
/// cube's first mip, so a reflection that meets nothing on the screen reads the trees and the hills around it rather
/// than the sky alone. A face is drawn again only where the camera moved from where it was drawn or the scene's
/// contents changed; where the sun turned or it has stood `REFRESH_FRAMES`, it is only lit again from the G-buffer it
/// keeps, which costs next to nothing, so a still view draws no geometry at all. A face never drawn at the probe's size
/// draws whole at once, and so does a face lit again. One left behind is drawn again a band a frame, `BANDS` of its
/// rows, into a spare G-buffer, each band offered to the frame's scheduler, and takes the face's place, lit, once
/// whole: a moving camera's faces never land whole on one frame.
pub struct SurroundingsCube {
  /// What culls and draws a face whole, and the camera it is lit by; and what culls and draws a band.
  cut: ShadowCut,
  band: ShadowCut,
  /// The faces' G-buffers, while the cube is in use.
  faces: Option<SurroundingsFaces>,
  /// The face this frame lights, or last lit, and the sun's direction for it.
  face: u32,
  to_sun: Vec3,
  /// Texels a side the cube's faces are, as the settings ask.
  size: u32,
  /// Whether this frame lights a face, and whether it draws its geometry whole first.
  is_due: bool,
  is_redrawn: bool,
  /// The band this frame draws of the face drawn again, as `commit` took it.
  band_due: Option<u32>,
  /// Whether it stands in the prefiltered cube's first mip this frame, drawn or kept: none while it rests, when the
  /// prefilter fills that mip with the sky.
  is_active: bool,
  /// Where each face was last drawn from and when it was last lit; none before it was drawn.
  drawn: [Option<FaceDrawn>; 6],
  /// The face drawn again band by band into the spare, while one is.
  redraw: Option<FaceRedraw>,
  /// Frames prepared since it was made.
  frame: u32,
  /// What this frame offered.
  offer: Option<(PeriodicTicket, CubeOffer)>,
}

/// The passes that draw a face whole, a band of one, and that light one again, as the timer names them.
static DRAW_PASSES: [&str; 3] = [
  SurroundingsPass::CULL_PASS,
  SurroundingsPass::DRAW_PASS,
  SurroundingsPass::RESOLVE_PASS,
];
static BAND_PASSES: [&str; 2] = ["surroundings band cull", "surroundings band"];
static LIGHT_PASSES: [&str; 1] = [SurroundingsPass::RESOLVE_PASS];

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

/// A face drawn again band by band: which, from where, over what the scene held at its first band, the bands drawn, and
/// the frames a band has waited.
#[derive(Clone, Copy)]
struct FaceRedraw {
  face: u32,
  position: Vec3,
  contents: usize,
  bands: u32,
  waited: u32,
}

/// What a frame offers: a face drawn whole or lit again, from where; or the next band of the face drawn again.
#[derive(Clone, Copy)]
enum CubeOffer {
  Face {
    face: u32,
    is_redrawn: bool,
    position: Vec3,
  },
  Band,
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
  /// Bands a face drawn again is drawn in, at most one a frame, and frames a band waits at most.
  const BANDS: u32 = 4;
  const BAND_WAIT_MOST: u32 = 2;
  /// Milliseconds a texel of a face drawn costs, and a face lit again, before the timer measured one.
  const DRAW_SEED: f32 = 1.1e-6;
  const LIGHT_SEED: f32 = 0.01;

  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64) -> Self {
    Self {
      cut: ShadowCut::new(device, view_layout, "surroundings", args_size),
      band: ShadowCut::new(device, view_layout, "surroundings band", args_size),
      faces: None,
      face: 5,
      to_sun: Vec3::Y,
      size: RenderReflectionProbe::Fine.get_size(),
      is_due: false,
      is_redrawn: false,
      band_due: None,
      is_active: false,
      drawn: [None; 6],
      redraw: None,
      frame: 0,
      offer: None,
    }
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
    self.band_due = None;
    self.offer = None;
    self.is_redrawn = false;
    self.is_active = false;
    self.drawn = [None; 6];
    self.redraw = None;
    self.faces = None;
  }

  /// Offers `scheduler` this frame's face, if one is due: a face drawn again band by band and now whole first, lit from
  /// where it was drawn; then one never drawn at this size, whole; then the next band of the face drawn again, or the
  /// first band of one the camera or the scene left behind, each after the last in turn, seen from where the
  /// viewport's camera stands; else one the sun turned from, then the one lit longest ago past `REFRESH_FRAMES`, lit
  /// again from where they were drawn.
  pub fn offer(
    &mut self,
    device: &wgpu::Device,
    frame: &ShadowFrame<'_>,
    probe: RenderReflectionProbe,
    scheduler: &mut PeriodicScheduler,
  ) {
    let size: u32 = probe.get_size();
    let drawn: u32 = Self::get_drawn_size(size);
    let camera: Vec3 = frame.camera.position;
    let to_sun: Vec3 = -frame.sun_direction;
    let texels: f32 = (drawn as f32).powi(2);

    self.frame = self.frame.wrapping_add(1);
    self.is_active = true;
    self.is_due = false;
    self.is_redrawn = false;
    self.band_due = None;
    self.offer = None;

    if self.faces.as_ref().is_none_or(|faces| faces.drawn != drawn) {
      self.faces = Some(SurroundingsFaces::new(device, drawn));
      self.drawn = [None; 6];
      self.redraw = None;
    }

    // A face drawn whole into the spare takes its place, and is lit from it at once.
    if let Some(redraw) = self.redraw.filter(|redraw| redraw.bands == Self::BANDS) {
      if let Some(faces) = &mut self.faces {
        faces.swap(redraw.face);
      }

      self.redraw = None;
      self.drawn[redraw.face as usize] = Some(FaceDrawn {
        position: redraw.position,
        contents: redraw.contents,
        size,
        to_sun,
        lit: self.frame,
      });
      self.offer_face(
        scheduler,
        (redraw.face, false, redraw.position),
        (&LIGHT_PASSES, Self::LIGHT_SEED, 1.0),
      );

      return;
    }

    let order = || (1..=6).map(|step| (self.face + step) % 6);

    if let Some(face) = order().find(|face| self.drawn[*face as usize].is_none_or(|it| it.size != size)) {
      self.offer_face(scheduler, (face, true, camera), (&DRAW_PASSES, Self::DRAW_SEED, texels));

      return;
    }

    let moved: Option<u32> = order().find(|face| {
      self.drawn[*face as usize].is_some_and(|it: FaceDrawn| {
        it.position.distance(camera) > Self::MOVED
          || frame.scene.is_changed_since(it.contents, |sphere| {
            Self::create_view(*face, it.position).reaches(sphere)
          })
      })
    });
    let scene = frame.scene;
    // A face's redraw started again where what it draws changed meanwhile.
    let redraw: Option<FaceRedraw> = self
      .redraw
      .filter(|redraw| {
        !scene.is_changed_since(redraw.contents, |sphere| {
          Self::create_view(redraw.face, redraw.position).reaches(sphere)
        })
      })
      .or_else(|| {
        moved.map(|face| FaceRedraw {
          face,
          position: camera,
          contents: scene.get_contents(),
          bands: 0,
          waited: 0,
        })
      });

    self.redraw = redraw;

    if let Some(redraw) = redraw {
      let job: PeriodicJob = PeriodicJob::new_waiting(
        &BAND_PASSES,
        Self::DRAW_SEED,
        texels / Self::BANDS as f32,
        (redraw.waited, Self::BAND_WAIT_MOST),
      );

      self.offer = Some((scheduler.offer(job), CubeOffer::Band));

      return;
    }

    let age = |face: u32| self.drawn[face as usize].map_or(0, |it| self.frame.wrapping_sub(it.lit));
    let turned: Option<u32> = order()
      .find(|face| self.drawn[*face as usize].is_some_and(|it: FaceDrawn| it.to_sun.dot(to_sun) < Self::SUN_TURNED));
    let aged: Option<u32> = (0..6)
      .filter(|face| age(*face) >= Self::REFRESH_FRAMES)
      .max_by_key(|face| age(*face));

    if let Some(face) = turned.or(aged) {
      let position: Vec3 = self.drawn[face as usize].map_or(camera, |it| it.position);

      self.offer_face(
        scheduler,
        (face, false, position),
        (&LIGHT_PASSES, Self::LIGHT_SEED, 1.0),
      );
    }
  }

  /// Readies what `scheduler` granted of this frame's offer: a face's camera, looking down its axis with the up a cube
  /// texture's face lays its texels by, or a band's, the same narrowed to its rows; a band not granted waits a frame
  /// more. `encoder` takes the copies a list growing makes.
  pub fn commit(
    &mut self,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    frame: &ShadowFrame<'_>,
    probe: RenderReflectionProbe,
    scheduler: &PeriodicScheduler,
  ) {
    let Some((ticket, offer)) = self.offer.take() else {
      return;
    };
    let is_granted: bool = scheduler.is_granted(ticket);
    let size: u32 = probe.get_size();
    let drawn: u32 = Self::get_drawn_size(size);
    let scene = frame.scene;
    let list_bytes: u64 = (scene.cull.get_list_capacity().max(1) as u64) * 8;

    match offer {
      CubeOffer::Band => {
        let Some(redraw) = self.redraw.as_mut() else {
          return;
        };

        if !is_granted {
          redraw.waited += 1;

          return;
        }

        let [x, y, width, height] = Self::get_band_region(redraw.bands, drawn);
        let whole: CameraView = Self::create_view(redraw.face, redraw.position);
        let band: CameraView = whole.narrow_to(
          Vec2::new(0.0, 1.0 - (2 * y + height) as f32 / drawn as f32),
          Vec2::new(2.0, 2.0 * height as f32 / drawn as f32),
        );

        self.band.lists.reserve(device, encoder, list_bytes);
        scene.cull.reset_args(encoder, &self.band.args);
        self.band.view.write(
          encoder,
          &CameraUniform::new(
            &band,
            RenderRect {
              x: x as i32,
              y: y as i32,
              width,
              height,
            },
            frame.switches,
          ),
        );
        self.band_due = Some(redraw.bands);
        redraw.bands += 1;
        redraw.waited = 0;
      }
      CubeOffer::Face {
        face,
        is_redrawn,
        position,
      } => {
        if !is_granted {
          return;
        }

        let to_sun: Vec3 = -frame.sun_direction;

        if is_redrawn {
          self.cut.lists.reserve(device, encoder, list_bytes);
          scene.cull.reset_args(encoder, &self.cut.args);
        }

        self.cut.view.write(
          encoder,
          &CameraUniform::new(
            &Self::create_view(face, position),
            RenderRect {
              x: 0,
              y: 0,
              width: drawn,
              height: drawn,
            },
            frame.switches,
          ),
        );

        self.face = face;
        self.to_sun = to_sun;
        self.size = size;
        self.is_due = true;
        self.is_redrawn = is_redrawn;
        self.drawn[face as usize] = Some(FaceDrawn {
          position,
          contents: match self.drawn[face as usize] {
            Some(it) if !is_redrawn => it.contents,
            _ => scene.get_contents(),
          },
          size,
          to_sun,
          lit: self.frame,
        });
      }
    }
  }

  /// Forgets that this frame drew its face, where its frame was not submitted: it draws again, and a face drawn band by
  /// band starts again.
  pub fn abandon_frame(&mut self) {
    if self.is_due {
      self.is_due = false;
      self.drawn[self.face as usize] = None;
    }

    if self.band_due.take().is_some() {
      self.redraw = None;
    }
  }

  /// A band's region of a face `drawn` texels across: every column of its rows, the bands together covering every row.
  fn get_band_region(band: u32, drawn: u32) -> [u32; 4] {
    let (first, last): (u32, u32) = (band * drawn / Self::BANDS, (band + 1) * drawn / Self::BANDS);

    [0, first, drawn, last - first]
  }

  fn offer_face(
    &mut self,
    scheduler: &mut PeriodicScheduler,
    (face, is_redrawn, position): (u32, bool, Vec3),
    (passes, seed, units): (&'static [&'static str], f32, f32),
  ) {
    let ticket: PeriodicTicket = scheduler.offer(PeriodicJob::new_now(passes, seed, units));

    self.offer = Some((
      ticket,
      CubeOffer::Face {
        face,
        is_redrawn,
        position,
      },
    ));
  }

  /// A face's camera from `position`, looking down its axis with the up a cube texture's face lays its texels by.
  fn create_view(face: u32, position: Vec3) -> CameraView {
    let (forward, up): (Vec3, Vec3) = match face {
      0 => (Vec3::X, Vec3::Y),
      1 => (Vec3::NEG_X, Vec3::Y),
      2 => (Vec3::Y, Vec3::NEG_Z),
      3 => (Vec3::NEG_Y, Vec3::Z),
      4 => (Vec3::Z, Vec3::Y),
      _ => (Vec3::NEG_Z, Vec3::Y),
    };
    let look: Mat4 = glam::camera::rh::view::look_at_mat4(position, position + forward, up);

    CameraView::new(position, look, 90.0, 1.0, Self::NEAR, Self::FAR)
  }

  /// Declares this frame's band of the face drawn again, into the spare G-buffer; then the face's cull and its draw into
  /// the G-buffer it keeps where it is drawn whole, and its resolve into `faces`, the prefiltered cube's first mip, where
  /// this frame lights one.
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
    let Some(kept) = &self.faces else {
      return;
    };
    let size: u32 = self.size;
    let drawn: u32 = kept.drawn;
    if let Some(band) = self.band_due {
      let targets: [GraphTexture; 4] = Self::import_layer((&mut *graph, &mut *bindings), kept.get_spare());

      self.add_draw_passes(
        (&mut *graph, &mut *bindings),
        passes,
        scene,
        (params, textures),
        (&self.band, &BAND_PASSES, Some(band)),
        targets,
      );
    }

    if !self.is_due {
      return;
    }

    let [albedo, normal, material, depth]: [GraphTexture; 4] =
      Self::import_layer((&mut *graph, &mut *bindings), kept.get_face(self.face));

    if self.is_redrawn {
      self.add_draw_passes(
        (&mut *graph, &mut *bindings),
        passes,
        scene,
        (params, textures),
        (&self.cut, &DRAW_PASSES, None),
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
        passes.surroundings.record(context, &self.cut.view, size, &parameters);
      });
  }

  /// Declares a cull by `cut` and its draw into a G-buffer, `targets` its albedo, normal, material and depth, under the
  /// cull's and draw's pass names: the whole face, cleared, or `band` of it, the first band clearing the rest.
  fn add_draw_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    scene: &StaticSceneHandles,
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
    (cut, names, band): (&'a ShadowCut, &'static [&'static str], Option<u32>),
    [albedo, normal, material, depth]: [GraphTexture; 4],
  ) {
    let (cull_pass, draw_pass): (&'static str, &'static str) = (names[0], names[1]);
    let args = bindings.import_buffer(graph, draw_pass, &cut.args);
    let lists = bindings.import_buffer(graph, cull_pass, cut.lists.get_buffer());
    let cull: StaticCullParameters = scene.get_cull_parameters(StorageArrayMut::new(lists), StorageArrayMut::new(args));
    let layouts: [StaticDrawParameters; StaticLayout::COUNT] = scene.get_layout_draws(StorageArray::new(lists));
    let draws: StaticDraws = StaticDraws {
      layouts,
      impostors: scene.get_camera_draws().impostors,
    };
    let drawn: u32 = self.faces.as_ref().map_or(0, |faces| faces.drawn);
    let region: [u32; 4] = band.map_or([0, 0, drawn, drawn], |band| Self::get_band_region(band, drawn));
    let (color_load, depth_load): (wgpu::LoadOp<wgpu::Color>, wgpu::LoadOp<f32>) = match band {
      Some(band) if band > 0 => (wgpu::LoadOp::Load, wgpu::LoadOp::Load),
      _ => (wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), wgpu::LoadOp::Clear(0.0)),
    };
    // The G-buffer pass writes a motion and a relief target too, which nothing reads of a face.
    let motion: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "surroundings motion",
      drawn,
      drawn,
      ViewTargets::MOTION,
    ));
    let relief: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "surroundings relief",
      drawn,
      drawn,
      ViewTargets::RELIEF,
    ));

    // The shadow's cull: every tree row as a mesh whatever its level of detail, no water, nothing hidden early.
    graph
      .add_compute_pass(cull_pass)
      .parameters(&cull)
      .record(move |context| {
        passes
          .cull
          .record_shadow(context, &cut.view, &cull, params, ShadowCull::Cascade);
      });

    let builder = [albedo, normal, material]
      .into_iter()
      .fold(graph.add_raster_pass(draw_pass), |builder, texture| {
        builder.color(GraphColorAttachment::new(texture, color_load))
      })
      .color(GraphColorAttachment::new(
        motion,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .color(GraphColorAttachment::new(
        relief,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .depth(GraphDepthAttachment::new(depth, depth_load))
      .buffer(args, GraphBufferAccess::Indirect);

    StaticDraws::declare_layouts(builder, &layouts).record(move |context| {
      let args: &wgpu::Buffer = context.get_buffer(args);

      StaticShadowPass::set_region(context, region);
      passes
        .gbuffer
        .record(context, (&cut.view, &draws, textures), args, (false, false));
    });
  }

  /// A layer of the faces' G-buffers as the frame's graph knows it: its albedo, normal, material and depth.
  fn import_layer<'a>(
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    layer: &'a [wgpu::TextureView; 4],
  ) -> [GraphTexture; 4] {
    std::array::from_fn(|target| {
      bindings.import_view(&mut *graph, SurroundingsFaces::FORMATS[target].0, &layer[target])
    })
  }
}

impl ViewFeature for SurroundingsCube {
  /// The faces' G-buffers and their spare, while the cube is in use.
  fn get_memory(&self) -> u64 {
    self.faces.as_ref().map_or(0, SurroundingsFaces::get_memory)
  }

  fn release(&mut self) {
    self.rest();
  }
}
