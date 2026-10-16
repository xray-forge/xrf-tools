use std::collections::{HashMap, HashSet};

use glam::{Vec3, Vec4};
use xrf_math::EPS_S;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, ProxyHandle,
  StorageArray, StorageArrayMut,
};
use xrf_visual::{LightDescription, LightKind};

use crate::camera::camera_view::CameraView;
use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_rect::RenderRect;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::lighting::light_basis::LightBasis;
use crate::lighting::light_shadow_size::{
  LIGHT_SHADOW_MIN_SIZE, LIGHT_SHADOW_POINT_CONE, LIGHT_SHADOW_POINT_FACES, LIGHT_SHADOW_WIDENING,
  to_light_shadow_size, to_light_shadow_tile_size,
};
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::shadow_casters::ShadowCasters;
use crate::pass::shadow_cull::ShadowCull;
use crate::pass::shadow_tile_restore_parameters::ShadowTileRestoreParameters;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_uniform::StaticCullUniform;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::grass_shadow_casters::GrassShadowCasters;
use crate::scene::level::light_shadow_atlas::LightShadowAtlas;
use crate::scene::level::light_shadow_entry::LightShadowEntry;
use crate::scene::level::light_shadow_face::LightShadowFace;
use crate::scene::level::light_shadow_set::LightShadowSet;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::level::shadow_tile::ShadowTile;
use crate::scene::level::shadow_tile_allocator::ShadowTileAllocator;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::static_cull_scratch::StaticCullScratch;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;

/// Shadow faces offered at most in one frame, each with a slot of its own in the lists and the draw arguments: the most
/// copies of the visible list the static scene admits an item by.
const FACE_BUDGET: usize = StaticCullScratch::MOST_LIST_COPIES as usize;

/// The passes that draw a frame's faces, as the timer names them.
static FACE_PASSES: [&str; 4] = [
  "light shadow arguments",
  "light shadow cull",
  "light shadows",
  "light shadow static",
];

/// A face due this frame as its passes take it: its slot, its square and how it is drawn, its cull and its draws.
type FaceSlot = (
  usize,
  (ShadowTile, FaceDraw, bool),
  StaticCullParameters,
  [StaticDrawParameters; StaticLayout::COUNT],
);

/// How a face due this frame is drawn.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaceDraw {
  /// Every caster into its tile, cleared first: while nothing sways in its light.
  Whole,
  /// What stands still into its static tile, then the tile restored from it and the trees drawn over.
  Static,
  /// Its tile restored from its static tile and the trees drawn over.
  Swaying,
}

/// Milliseconds a texel of a face costs before the timer measured one.
const FACE_SEED: f32 = 7.6e-7;

/// Frames a face waits at most where it holds nothing yet or the scene changed under it, which leaves its light unlit
/// or wrong; and where it is drawn again only for what sways in it. A face whose light moved waits none: its shadow
/// would trail the light.
const STALE_WAIT_MOST: u32 = 2;
const SWAY_WAIT_MOST: u32 = 8;

/// How far a light's wanted size may stray from the square it was asked at before it is asked at another.
const GROW: f32 = 1.5;
const SHRINK: f32 = 0.66;

/// Where a face's projection starts where the light gives none: `light::virtual_size`'s default.
const DEFAULT_NEAR: f32 = 0.1;

/// Metres from the eye within which a light casts the grass's shadow, as Screen Space Shaders' grass shadows for
/// non-directional lights default to: each of its faces draws the cards again as often as the trees' sway, so a few
/// nearest lights pay it.
const GRASS_LIGHT_DISTANCE: f32 = 30.0;

/// How far a tuft's tip leans for the wind's amplitude, in metres, which a grassed face is drawn again for as a tree
/// is.
const GRASS_LEAN: f32 = 0.3;

/// A face wanting a draw: what orders it (out of view, then not stale, then how near or how long ago drawn), its light,
/// whether of the light's next set, which face, and the frames it may wait, none where it draws at once.
type FaceCandidate = ((bool, bool, f32), ProxyHandle<LightDescription>, bool, usize, u32);

/// The local lights' shadows: a square of an atlas a face, sized as the engine sizes its maps, drawn once and kept
/// while the scene it casts from stays, a few faces a frame as the frame's scheduler grants them by their texels, the
/// nearest lights first; a face over swaying trees is drawn again once a sway interval, where the lean shows at its
/// texels. A light lights only once its faces are drawn; one asked at another size keeps its old faces until its new
/// ones are. While trees sway in a light's reach, each of its faces keeps what stands still in the same tile of a
/// static atlas, drawn only when the scene changes under it or its light moves; a sway's draw restores the tile from it
/// and draws the trees over. Room is made from the lights out of view longest, and where there is still none, a light
/// is asked at smaller squares. A light out of view but standing near the camera is kept ready, so a turn finds it
/// drawn: it holds the faces it has, takes room only where some is free, at smaller squares where little is, and its
/// faces are drawn after those in view. Each face drawn in a frame has a slot of its own in the lists and the draw arguments,
/// so every face is culled before any is drawn, and all are drawn in one render pass. The atlas is held only while a
/// view of the scene casts the lights' shadows; a texel of nothing is bound in its place otherwise.
pub struct LevelLightShadows {
  /// The atlas, while a view casts the lights' shadows, and what is bound in its place while none does.
  atlas: Option<wgpu::TextureView>,
  none: wgpu::TextureView,
  /// Compares a depth with the atlas between four texels, lit where the stored depth, reversed, is no nearer.
  sampler: wgpu::Sampler,
  /// Whether a view cast the lights' shadows this frame.
  is_cast: bool,
  /// What stands still of the swaying lights' faces, tile for tile as the atlas, while any light asked for sways.
  statics: Option<wgpu::TextureView>,
  /// Whether a light asked for this frame has trees swaying in its reach.
  is_swaying: bool,
  allocator: ShadowTileAllocator,
  entries: HashMap<ProxyHandle<LightDescription>, LightShadowEntry>,
  frame: u64,
  /// The faces wanting a draw this frame, as their light, set and face, behind what orders them: stale ones first,
  /// nearest lights first, then the ones over what sways, drawn longest ago first.
  candidates: Vec<FaceCandidate>,
  /// The faces offered this frame, by their light, set and face, with their ticket.
  queue: Vec<(ProxyHandle<LightDescription>, bool, usize, PeriodicTicket)>,
  /// The faces this frame draws, by their slot, their square and how, and whether the grass's cards are drawn over,
  /// as `prepare` readied them.
  due: Vec<(usize, ShadowTile, FaceDraw, bool)>,
  /// The lights whose faces this frame draws, by their light and square.
  drawn: Vec<(ProxyHandle<LightDescription>, ShadowTile)>,
  /// A face's camera, a slot each.
  views: Vec<ViewBinding>,
  /// Every slot's visible list, `region` bytes apart.
  lists: GrowableBuffer,
  region: u64,
  /// Every slot's draw arguments, `args_stride` bytes apart, each a cull's arguments long.
  args: wgpu::Buffer,
  args_stride: u64,
  /// What a slot's range of a buffer starts at a multiple of.
  alignment: u64,
}

impl LevelLightShadows {
  /// Texels the light shadow atlas is across.
  pub const ATLAS_SIZE: u32 = 4096;

  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout) -> Self {
    let alignment: u64 = u64::from(device.limits().min_storage_buffer_offset_alignment);
    let args_stride: u64 = StaticCullScratch::ARGS_SIZE.next_multiple_of(alignment);
    Self {
      atlas: None,
      none: Self::create_atlas(device, "light shadow atlas none", 1),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("light shadow atlas"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        compare: Some(wgpu::CompareFunction::GreaterEqual),
        ..Default::default()
      }),
      is_cast: false,
      statics: None,
      is_swaying: false,
      allocator: ShadowTileAllocator::new(Self::ATLAS_SIZE, LIGHT_SHADOW_MIN_SIZE),
      entries: HashMap::new(),
      frame: 0,
      candidates: Vec::new(),
      queue: Vec::new(),
      due: Vec::new(),
      drawn: Vec::new(),
      views: (0..FACE_BUDGET)
        .map(|_| ViewBinding::new(device, view_layout))
        .collect(),
      lists: GrowableBuffer::new(device, "light shadow lists", wgpu::BufferUsages::STORAGE),
      region: 0,
      args: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("light shadow draw arguments"),
        size: args_stride * FACE_BUDGET as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      args_stride,
      alignment,
    }
  }

  /// Texels of the atlas the lights' faces hold, of its whole.
  pub fn get_atlas_use(&self) -> RenderPoolUse {
    RenderPoolUse {
      used: self.allocator.get_used().min(u64::from(u32::MAX)) as u32,
      capacity: Self::ATLAS_SIZE * Self::ATLAS_SIZE,
    }
  }

  /// The atlas the lights sample, or a texel of nothing while none is held, with the sampler comparing it.
  pub fn get_atlas(&self) -> LightShadowAtlas<'_> {
    LightShadowAtlas {
      view: self.atlas.as_ref().unwrap_or(&self.none),
      sampler: &self.sampler,
    }
  }

  /// Takes a view casting the lights' shadows this frame, which holds the atlas.
  pub fn cast(&mut self) {
    self.is_cast = true;
  }

  /// Starts a frame's asks.
  pub fn begin(&mut self) {
    self.frame += 1;
    self.candidates.clear();
    self.queue.clear();
    self.is_swaying = false;
    self.is_cast = false;
  }

  /// Takes a shadowed light in view this frame, nearest first, or one out of view kept ready (`is_in_view` false), after
  /// every one in view: gives it faces where it has none at the size it wants, and puts forward those of its faces
  /// wanting a draw. One kept ready keeps the faces it has, and takes new ones only from free room.
  #[allow(clippy::too_many_arguments)]
  pub fn ask(
    &mut self,
    handle: ProxyHandle<LightDescription>,
    light: &LightDescription,
    basis: &LightBasis,
    eye: Vec3,
    forward: Vec3,
    color: Vec3,
    statics: &StaticScene,
    (sway, grass_reach): (&ShadowSway<'_>, Option<f32>),
    is_in_view: bool,
  ) {
    let is_spot: bool = matches!(light.kind, LightKind::Spot);
    let spatial: Vec4 = basis.get_spatial_sphere(light);
    let distance: f32 = (eye.distance(spatial.truncate()) - spatial.w).max(0.0);
    let duel: f32 = if is_spot {
      1.0 - 0.5 * forward.dot(basis.direction)
    } else {
      1.0
    };
    let cone: f32 = if is_spot { light.cone } else { LIGHT_SHADOW_POINT_CONE };
    let wanted: f32 = to_light_shadow_size(light.range, distance, LightBasis::to_light_intensity(color), duel, cone);
    let entry: &mut LightShadowEntry = self.entries.entry(handle).or_default();

    // Kept ready out of view once a frame, and never over a view's ask.
    if !is_in_view {
      if entry.seen == self.frame || entry.warmed == self.frame {
        return;
      }

      entry.warmed = self.frame;
    }

    let is_asked: bool = is_in_view && entry.seen == self.frame;
    let current: Option<u32> = entry.next.as_ref().or(entry.shown.as_ref()).map(|set| set.size);
    let asked: u32 = match current {
      Some(size) if !is_in_view || (wanted <= size as f32 * GROW && wanted >= size as f32 * SHRINK) => size,
      _ => to_light_shadow_tile_size(wanted),
    };

    // Another view of the scene asked for it this frame already: the size the nearer one wants stands.
    if is_asked && current.is_some_and(|size| asked <= size) {
      return;
    }

    if is_in_view {
      entry.seen = self.frame;
    }

    // Back at the size it shows while another was being drawn: the shown faces stand, and the other goes.
    if entry.shown.as_ref().is_some_and(|set| set.size == asked) {
      if let Some(next) = entry.next.take() {
        Self::release_set(&mut self.allocator, next);
      }
    } else if entry.next.as_ref().is_none_or(|set| set.size != asked) {
      if let Some(next) = self.entries.get_mut(&handle).and_then(|entry| entry.next.take()) {
        Self::release_set(&mut self.allocator, next);
      }

      let set: Option<LightShadowSet> = self.create_set(light, basis, asked, is_in_view);

      if let Some(entry) = self.entries.get_mut(&handle) {
        entry.next = set;
      }
    }

    let Some(entry) = self.entries.get_mut(&handle) else {
      return;
    };

    let near: f32 = entry
      .next
      .as_ref()
      .or(entry.shown.as_ref())
      .map_or(DEFAULT_NEAR, |set| set.near);

    // Only an item coming or going within the light's reach changes what its faces hold or what sways in it.
    let contents: usize = statics.get_contents();
    let touches = |sphere: Vec4| spatial.truncate().distance(sphere.truncate()) < spatial.w + sphere.w;

    if entry
      .swaying
      .is_none_or(|(at, _)| statics.is_changed_since(at, touches))
    {
      entry.swaying = Some((
        contents,
        Self::measure_swaying(basis.position, spatial, near, sway.places),
      ));
    }

    // The cards stand about the eye: a light near it whose reach meets them casts their shadow.
    let is_grassed: bool = distance < GRASS_LIGHT_DISTANCE
      && grass_reach.is_some_and(|reach| eye.distance(spatial.truncate()) < spatial.w + reach);
    let swaying: f32 =
      entry
        .swaying
        .map_or(0.0, |(_, swaying)| swaying)
        .max(if is_grassed { GRASS_LEAN / near } else { 0.0 });

    entry.is_grassed = is_grassed;

    self.is_swaying |= sway.amplitude * swaying > 0.0;
    // How wide a face's texels are, for each metre from the light, times the square's side.
    let spread: f32 = 2.0 * ((cone + LIGHT_SHADOW_WIDENING) * 0.5).tan();

    for (is_next, set) in [(false, &mut entry.shown), (true, &mut entry.next)] {
      if let Some(set) = set {
        for (face, state) in set.faces.iter_mut().enumerate() {
          // A light a motion carries has moved off where its faces look from: each looks from where it stands now,
          // and is drawn again as a swaying one is, lighting with what it holds meanwhile.
          let is_moved: bool = state.view.position != basis.position;

          if is_moved {
            state.view = Self::to_face_view(light, basis, face, set.near, set.far);
            state.drawn_static = None;
          }

          // Out of view, nothing it holds shows: it waits as long as a swaying face.
          let (is_away, stale_wait, moved_wait): (bool, u32, u32) = if is_in_view {
            (false, STALE_WAIT_MOST, 0)
          } else {
            (true, SWAY_WAIT_MOST, SWAY_WAIT_MOST)
          };

          if state.drawn.is_none_or(|drawn| statics.is_changed_since(drawn, touches)) || state.is_grassed != is_grassed
          {
            state.drawn_static = None;
            self
              .candidates
              .push(((is_away, false, distance), handle, is_next, face, stale_wait));
          } else if is_moved {
            self
              .candidates
              .push(((is_away, true, state.drawn_at), handle, is_next, face, moved_wait));
          } else if sway.is_redrawn(swaying, spread / state.tile.size as f32, state.drawn_at) {
            self
              .candidates
              .push(((is_away, true, state.drawn_at), handle, is_next, face, SWAY_WAIT_MOST));
          }
        }
      }
    }
  }

  /// Offers `scheduler` the faces wanting a draw this frame, the nearest lights' first, each once however many views
  /// asked for it, by their texels: one holding nothing yet or changed under within `STALE_WAIT_MOST` frames, one
  /// swaying within `SWAY_WAIT_MOST`, one its light carried at once.
  pub fn finish(&mut self, scheduler: &mut PeriodicScheduler) {
    self
      .candidates
      .sort_by(|a, b| (a.0.0, a.0.1).cmp(&(b.0.0, b.0.1)).then(a.0.2.total_cmp(&b.0.2)));
    let mut queued: HashSet<(ProxyHandle<LightDescription>, bool, usize)> = HashSet::new();
    let faces: Vec<(ProxyHandle<LightDescription>, bool, usize, u32)> = self
      .candidates
      .iter()
      .filter(|(_, index, is_next, face, _)| queued.insert((*index, *is_next, *face)))
      .take(FACE_BUDGET)
      .map(|(_, index, is_next, face, wait_most)| (*index, *is_next, *face, *wait_most))
      .collect();

    self.queue.clear();

    for (index, is_next, face, wait_most) in faces {
      let Some(state) = self.get_face(index, is_next, face) else {
        continue;
      };
      let texels: f32 = (state.tile.size as f32).powi(2);
      let job: PeriodicJob = if wait_most == 0 {
        PeriodicJob::new_now(&FACE_PASSES, FACE_SEED, texels)
      } else {
        PeriodicJob::new_waiting(&FACE_PASSES, FACE_SEED, texels, (state.waited, wait_most))
      };

      self.queue.push((index, is_next, face, scheduler.offer(job)));
    }
  }

  /// The faces a light lights with this frame, every one drawn, or none where it has none whole.
  pub fn get_set(&self, handle: ProxyHandle<LightDescription>) -> Option<&LightShadowSet> {
    self.entries.get(&handle)?.shown.as_ref().filter(|set| set.is_drawn())
  }

  /// Readies the faces `scheduler` granted this frame to be culled into their slots and drawn into their squares of the
  /// atlas, one not granted waiting a frame more, and lets a light's new faces take the place of its old once all are
  /// drawn; `encoder` takes the copies a list growing makes.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    frame: &ShadowFrame<'_>,
    scheduler: &PeriodicScheduler,
  ) {
    let scene = frame.scene;
    let contents: usize = scene.get_contents();

    self.due.clear();
    self.drawn.clear();
    self.hold_atlas(device);
    self.hold_statics(device);

    if self.queue.is_empty() {
      return;
    }

    self.region = scene.cull.get_list_bytes().next_multiple_of(self.alignment);
    self.lists.reserve(device, encoder, self.region * FACE_BUDGET as u64);

    for (index, is_next, face, ticket) in std::mem::take(&mut self.queue) {
      let slot: usize = self.due.len();
      let is_granted: bool = scheduler.is_granted(ticket);
      let is_split: bool = self.statics.is_some()
        && self
          .entries
          .get(&index)
          .and_then(|entry| entry.swaying)
          .is_some_and(|(_, swaying)| swaying > 0.0);
      let is_grassed: bool = self.entries.get(&index).is_some_and(|entry| entry.is_grassed);
      let Some(state) = self
        .entries
        .get_mut(&index)
        .and_then(|entry| {
          if is_next {
            entry.next.as_mut()
          } else {
            entry.shown.as_mut()
          }
        })
        .and_then(|set| set.faces.get_mut(face))
      else {
        continue;
      };

      if !is_granted {
        state.waited += 1;

        continue;
      }

      state.waited = 0;

      let view: &ViewBinding = &self.views[slot];

      view.write(
        encoder,
        &CameraUniform::new(
          &state.view,
          RenderRect {
            x: 0,
            y: 0,
            width: state.tile.size,
            height: state.tile.size,
          },
          Vec4::ZERO,
        ),
      );
      let draw: FaceDraw = match (is_split, state.drawn_static) {
        (false, _) => FaceDraw::Whole,
        (true, None) => FaceDraw::Static,
        (true, Some(_)) => FaceDraw::Swaying,
      };

      state.drawn = Some(contents);
      state.drawn_at = frame.sway.time;
      state.is_grassed = is_grassed;

      if draw == FaceDraw::Static {
        state.drawn_static = Some(contents);
      }

      self.due.push((slot, state.tile, draw, is_grassed));
      self.drawn.push((index, state.tile));
    }

    // A light's new faces, all drawn, take the place of its old.
    for entry in self.entries.values_mut() {
      if entry.next.as_ref().is_some_and(LightShadowSet::is_drawn) {
        if let Some(shown) = entry.shown.take() {
          Self::release_set(&mut self.allocator, shown);
        }

        entry.shown = entry.next.take();
      }
    }
  }

  /// Declares the faces this frame's `prepare` readied: each slot's arguments started from those a cull starts with,
  /// every face culled into its slot in one compute pass, then every face drawn into its square in one render pass, the
  /// `grass`'s cards over those whose light casts from it.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    scene: &StaticSceneHandles,
    (params, textures): (&'a StaticCullUniform, &'a wgpu::BindGroup),
    grass: Option<GrassShadowCasters<'a>>,
  ) {
    if self.due.is_empty() {
      return;
    }

    let template: GraphBuffer = scene.args_template;
    let args: GraphBuffer = bindings.import_buffer(graph, "light shadow draw arguments", &self.args);
    let lists: GraphBuffer = bindings.import_buffer(graph, "light shadow lists", self.lists.get_buffer());
    let Some(atlas) = &self.atlas else {
      return;
    };
    let atlas: GraphTexture = bindings.import_view(graph, "light shadow atlas", atlas);
    // Each slot's cull into its range of the lists and the arguments, and its draws of that range.
    let slots: Vec<FaceSlot> = self
      .due
      .iter()
      .map(|(slot, tile, draw, is_grassed)| {
        let (list_offset, args_offset): (u64, u64) = (*slot as u64 * self.region, *slot as u64 * self.args_stride);

        (
          *slot,
          (*tile, *draw, *is_grassed && grass.is_some()),
          scene.get_cull_parameters(
            StorageArrayMut::new_range(lists, list_offset, self.region),
            StorageArrayMut::new_range(args, args_offset, StaticCullScratch::ARGS_SIZE),
          ),
          scene.get_layout_draws(StorageArray::new_range(lists, list_offset, self.region)),
        )
      })
      .collect();

    graph
      .add_encoder_pass(FACE_PASSES[0])
      .buffer(template, GraphBufferAccess::CopySource)
      .buffer(args, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let (template, args) = (context.get_buffer(template), context.get_buffer(args));

        for (slot, ..) in &self.due {
          context.get_encoder().copy_buffer_to_buffer(
            template,
            0,
            args,
            *slot as u64 * self.args_stride,
            StaticCullScratch::ARGS_SIZE,
          );
        }
      });
    slots
      .iter()
      .fold(graph.add_compute_pass(FACE_PASSES[1]), |builder, (_, _, cull, _)| {
        builder.parameters(cull)
      })
      .record({
        let culls: Vec<(usize, StaticCullParameters)> = slots.iter().map(|(slot, _, cull, _)| (*slot, *cull)).collect();

        move |context| {
          for (slot, cull) in &culls {
            passes
              .cull
              .record_shadow(context, &self.views[*slot], cull, params, ShadowCull::LightFace);
          }
        }
      });
    // What stands still of the faces whose static tiles are stale, into those tiles, before any tile is restored.
    let restore: Option<ShadowTileRestoreParameters> =
      self.statics.as_ref().map(|statics| ShadowTileRestoreParameters {
        static_atlas: bindings.import_view(&mut *graph, "light shadow static atlas", statics),
      });
    let statics: Vec<(usize, ShadowTile, [StaticDrawParameters; StaticLayout::COUNT])> = slots
      .iter()
      .filter(|(_, (_, draw, _), ..)| *draw == FaceDraw::Static)
      .map(|(slot, (tile, ..), _, layouts)| (*slot, *tile, *layouts))
      .collect();

    if let (Some(restore), false) = (restore, statics.is_empty()) {
      statics
        .iter()
        .fold(
          graph
            .add_raster_pass(FACE_PASSES[3])
            .depth(GraphDepthAttachment::new(restore.static_atlas, wgpu::LoadOp::Load))
            .buffer(args, GraphBufferAccess::Indirect),
          |builder, (.., layouts)| StaticDraws::declare_layouts(builder, layouts),
        )
        .record(move |context| {
          let args: &wgpu::Buffer = context.get_buffer(args);

          for (slot, tile, layouts) in &statics {
            passes.shadow.record_tile(
              context,
              (tile.get_region(), None),
              (&self.views[*slot], layouts, textures),
              (args, *slot as u64 * self.args_stride),
              ShadowCasters::Fixed,
            );
          }
        });
    }

    let is_restored: bool = slots.iter().any(|(_, (_, draw, _), ..)| *draw != FaceDraw::Whole);
    let grass: Option<GrassShadowCasters<'a>> =
      grass.filter(|_| slots.iter().any(|(_, (.., is_grassed), ..)| *is_grassed));
    let builder = slots.iter().fold(
      graph
        .add_raster_pass(FACE_PASSES[2])
        .depth(GraphDepthAttachment::new(atlas, wgpu::LoadOp::Load))
        .buffer(args, GraphBufferAccess::Indirect),
      |builder, (.., layouts)| StaticDraws::declare_layouts(builder, layouts),
    );
    let builder = match restore.filter(|_| is_restored) {
      Some(restore) => builder.parameters(&restore),
      None => builder,
    };
    let builder = match grass {
      Some(grass) => grass.declare(builder),
      None => builder,
    };

    builder.record(move |context| {
      let args: &wgpu::Buffer = context.get_buffer(args);

      for (slot, (tile, draw, is_grassed), _, layouts) in &slots {
        // A swaying light's face over what stands still, as kept; any other drawn whole.
        let (restored, casters): (Option<&ShadowTileRestoreParameters>, ShadowCasters) = match (draw, &restore) {
          (FaceDraw::Static | FaceDraw::Swaying, Some(restore)) => (Some(restore), ShadowCasters::Swaying),
          _ => (None, ShadowCasters::All),
        };

        passes.shadow.record_tile(
          context,
          (tile.get_region(), restored),
          (&self.views[*slot], layouts, textures),
          (args, *slot as u64 * self.args_stride),
          casters,
        );

        if let Some(grass) = grass.filter(|_| *is_grassed) {
          grass.record(context, &self.views[*slot]);
        }
      }
    });
  }

  /// Forgets that the faces this frame drew were drawn, where its frame was not submitted: each draws again, and its
  /// light waits for it.
  pub fn abandon_frame(&mut self) {
    for (index, tile) in std::mem::take(&mut self.drawn) {
      let Some(entry) = self.entries.get_mut(&index) else {
        continue;
      };

      for set in entry.shown.iter_mut().chain(entry.next.iter_mut()) {
        for face in set.faces.iter_mut().filter(|face| face.tile == tile) {
          face.drawn = None;
          face.drawn_static = None;
        }
      }
    }

    self.due.clear();
  }

  /// Makes the atlas while a view casts the lights' shadows, and lets it go, with every light's faces, once none does.
  fn hold_atlas(&mut self, device: &wgpu::Device) {
    match (self.is_cast, self.atlas.is_some()) {
      (true, false) => {
        self.atlas = Some(Self::create_atlas(device, "light shadow atlas", Self::ATLAS_SIZE));
      }
      (false, true) => {
        self.atlas = None;
        self.statics = None;
        self.entries.clear();
        self.allocator = ShadowTileAllocator::new(Self::ATLAS_SIZE, LIGHT_SHADOW_MIN_SIZE);
      }
      _ => {}
    }
  }

  /// Makes the static atlas while a light asked for this frame sways, and lets it go, with every face's static tile,
  /// once none does.
  fn hold_statics(&mut self, device: &wgpu::Device) {
    match (self.is_swaying, self.statics.is_some()) {
      (true, false) => {
        self.statics = Some(Self::create_atlas(
          device,
          "light shadow static atlas",
          Self::ATLAS_SIZE,
        ));
      }
      (false, true) => {
        self.statics = None;

        for set in self
          .entries
          .values_mut()
          .flat_map(|entry| entry.shown.iter_mut().chain(entry.next.iter_mut()))
        {
          set.faces.iter_mut().for_each(|face| face.drawn_static = None);
        }
      }
      _ => {}
    }
  }

  /// A face of a light's shown set or its next.
  fn get_face(&self, index: ProxyHandle<LightDescription>, is_next: bool, face: usize) -> Option<&LightShadowFace> {
    let entry: &LightShadowEntry = self.entries.get(&index)?;
    let set: &LightShadowSet = if is_next {
      entry.next.as_ref()
    } else {
      entry.shown.as_ref()
    }?;

    set.faces.get(face)
  }

  /// A light's faces at a size, room made from the lights out of view longest where it `may_evict`, and at smaller
  /// squares where there is still none.
  fn create_set(
    &mut self,
    light: &LightDescription,
    basis: &LightBasis,
    size: u32,
    may_evict: bool,
  ) -> Option<LightShadowSet> {
    let is_spot: bool = matches!(light.kind, LightKind::Spot);
    let count: usize = if is_spot { 1 } else { LIGHT_SHADOW_POINT_FACES.len() };
    let near: f32 = if light.near > 0.0 { light.near } else { DEFAULT_NEAR };
    let far: f32 = light.range + light.range_jitter + EPS_S;
    let mut side: u32 = size;

    loop {
      if let Some(tiles) = self.allocate(count, side) {
        let faces: Vec<LightShadowFace> = tiles
          .into_iter()
          .enumerate()
          .map(|(face, tile)| LightShadowFace {
            tile,
            view: Self::to_face_view(light, basis, face, near, far),
            drawn: None,
            drawn_static: None,
            drawn_at: 0.0,
            waited: 0,
            is_grassed: false,
          })
          .collect();

        return Some(LightShadowSet { size, near, far, faces });
      }

      if !(may_evict && self.evict_oldest()) {
        if side <= LIGHT_SHADOW_MIN_SIZE {
          return None;
        }

        side /= 2;
      }
    }
  }

  /// Squares for every face of a light, all or none.
  fn allocate(&mut self, count: usize, side: u32) -> Option<Vec<ShadowTile>> {
    let mut tiles: Vec<ShadowTile> = Vec::with_capacity(count);

    for _ in 0..count {
      match self.allocator.allocate(side) {
        Some(tile) => tiles.push(tile),
        None => {
          tiles.into_iter().for_each(|tile| self.allocator.release(tile));

          return None;
        }
      }
    }

    Some(tiles)
  }

  /// Lets the light out of view longest give its faces back; false where every light holding any is in view.
  fn evict_oldest(&mut self) -> bool {
    let Some(index) = self
      .entries
      .iter()
      .filter(|(_, entry)| entry.seen < self.frame && (entry.shown.is_some() || entry.next.is_some()))
      .min_by_key(|(_, entry)| entry.seen)
      .map(|(index, _)| *index)
    else {
      return false;
    };

    if let Some(entry) = self.entries.remove(&index) {
      entry
        .shown
        .into_iter()
        .chain(entry.next)
        .for_each(|set| Self::release_set(&mut self.allocator, set));
    }

    true
  }

  /// A depth atlas `size` texels a side the faces are drawn into and the lights sample.
  fn create_atlas(device: &wgpu::Device, label: &'static str, size: u32) -> wgpu::TextureView {
    device
      .create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
          width: size,
          height: size,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
      .create_view(&Default::default())
  }

  /// The most any swaying place a light's sphere reaches leans for each metre it stands from the light, by its reach
  /// over its nearest distance: what a face's texel, which widens with the distance, is weighed against.
  fn measure_swaying(position: Vec3, sphere: Vec4, near: f32, places: &[(Vec4, f32)]) -> f32 {
    places
      .iter()
      .filter(|(place, _)| sphere.truncate().distance(place.truncate()) < sphere.w + place.w)
      .map(|(place, reach)| reach / (position.distance(place.truncate()) - place.w).max(near))
      .fold(0.0, f32::max)
  }

  fn release_set(allocator: &mut ShadowTileAllocator, set: LightShadowSet) {
    set.faces.into_iter().for_each(|face| allocator.release(face.tile));
  }

  /// A face's camera, built as `compute_xf_spot` builds its own: at the light, down its direction or the face's axis,
  /// the cone widened, its depth reversed.
  fn to_face_view(light: &LightDescription, basis: &LightBasis, face: usize, near: f32, far: f32) -> CameraView {
    let (direction, up, cone): (Vec3, Vec3, f32) = match light.kind {
      LightKind::Spot => (basis.direction, basis.up, light.cone),
      LightKind::Point => {
        let (direction, up) = LIGHT_SHADOW_POINT_FACES[face];

        (direction, up, LIGHT_SHADOW_POINT_CONE)
      }
    };
    let view = glam::camera::rh::view::look_at_mat4(basis.position, basis.position + direction, up);

    CameraView::new(
      basis.position,
      view,
      (cone + LIGHT_SHADOW_WIDENING).to_degrees(),
      1.0,
      near,
      far,
    )
  }
}
