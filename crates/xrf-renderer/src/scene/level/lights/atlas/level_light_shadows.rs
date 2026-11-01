use std::collections::{HashMap, HashSet};

use glam::{Vec3, Vec4};
use xrf_math::EPS_S;
use xrf_renderer_core::{
  GraphBuffer, GraphBufferAccess, GraphDeclaration, GraphDepthAttachment, GraphTexture, ProxyHandle, StorageArray,
  StorageArrayMut,
};
use xrf_visual::{LightDescription, LightKind};

use crate::camera::camera_view::CameraView;
use crate::contract::report::frame::render_pool_use::RenderPoolUse;
use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::lighting::shadows::shadow_casters::ShadowCasters;
use crate::feature::lighting::shadows::shadow_tile_restore_parameters::ShadowTileRestoreParameters;
use crate::feature::scene::drawable::casters::drawable_kind_ref::DrawableKindRef;
use crate::feature::scene::drawable::casters::light_casters::LightCasters;
use crate::feature::scene::drawable::drawable_kinds::DrawableKinds;
use crate::feature::scene::statics::cull::shadow_cull::ShadowCull;
use crate::feature::scene::statics::cull::static_cull_parameters::StaticCullParameters;
use crate::feature::scene::statics::draw::static_draw_parameters::StaticDrawParameters;
use crate::feature::scene::statics::draw::static_draws::StaticDraws;
use crate::feature::scene::statics::draw::static_layout_bindings::StaticLayoutBindings;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::lighting::lamp::light_basis::LightBasis;
use crate::lighting::lamp::light_shadow_size::{
  LIGHT_SHADOW_MIN_SIZE, LIGHT_SHADOW_POINT_CONE, LIGHT_SHADOW_POINT_FACES, LIGHT_SHADOW_WIDENING,
  to_light_shadow_size, to_light_shadow_tile_size,
};
use crate::pass::build::indirect_args::IndirectArgs;
use crate::pass::view::camera_uniform::CameraUniform;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::lights::atlas::light_shadow_ask::LightShadowAsk;
use crate::scene::level::lights::atlas::light_shadow_atlas::LightShadowAtlas;
use crate::scene::level::lights::atlas::light_shadow_entry::LightShadowEntry;
use crate::scene::level::lights::atlas::light_shadow_face::LightShadowFace;
use crate::scene::level::lights::atlas::light_shadow_set::LightShadowSet;
use crate::scene::level::lights::atlas::shadow_tile::ShadowTile;
use crate::scene::level::lights::atlas::shadow_tile_allocator::ShadowTileAllocator;
use crate::scene::level::shadow::shadow_frame::ShadowFrame;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::static_scene::draw::static_cull_scratch::StaticCullScratch;
use crate::scene::static_scene::geometry::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;

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

/// Which of a light's two sets a face is of: the one it lights with, or the one drawn at another size to take its place.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum FaceSet {
  Shown,
  Next,
}

/// A face of a light: the light, which of its sets, which face.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct FaceKey {
  handle: ProxyHandle<LightDescription>,
  set: FaceSet,
  face: usize,
}

/// What orders the faces wanting a draw: in view before out of it, those holding nothing yet or changed under before
/// those moved or swaying (`is_current`), then by `by`: the nearest stale lights first, the others drawn longest ago.
#[derive(Clone, Copy, Debug)]
struct FaceOrder {
  is_away: bool,
  is_current: bool,
  by: f32,
}

/// A face wanting a draw: what orders it, which face, and the frames it may wait, none where it draws at once.
#[derive(Clone, Copy, Debug)]
struct FaceCandidate {
  order: FaceOrder,
  key: FaceKey,
  wait_most: u32,
}

/// A face offered to this frame's scheduler, by its ticket.
#[derive(Clone, Copy, Debug)]
struct FaceOffer {
  key: FaceKey,
  ticket: PeriodicTicket,
}

/// A face this frame draws: its slot, its square, how it is drawn and the drawable kinds whose casters go over it.
#[derive(Clone, Copy, Debug)]
struct FaceDue {
  slot: usize,
  tile: ShadowTile,
  draw: FaceDraw,
  casting: DrawableKinds,
}

/// A light's face this frame draws, by its light and square.
#[derive(Clone, Copy, Debug)]
struct FaceDrawn {
  handle: ProxyHandle<LightDescription>,
  tile: ShadowTile,
}

/// A face due this frame as its passes take it: what it draws, its cull and its draws.
#[derive(Clone, Copy)]
struct FaceSlot {
  due: FaceDue,
  cull: StaticCullParameters,
  layouts: [StaticDrawParameters; StaticLayout::COUNT],
}

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

/// The local lights' shadows: a square of an atlas a face, sized as the engine sizes its maps, drawn once and kept
/// while the scene it casts from stays, a few faces a frame as the frame's scheduler grants them, the nearest first.
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
  /// Whether a light asked for this frame has anything swaying in its reach: trees, or what a drawable kind leans.
  is_swaying: bool,
  allocator: ShadowTileAllocator,
  entries: HashMap<ProxyHandle<LightDescription>, LightShadowEntry>,
  frame: u64,
  /// The faces wanting a draw this frame: stale ones first, nearest lights first, then the ones over what sways, drawn
  /// longest ago first.
  candidates: Vec<FaceCandidate>,
  /// The faces offered this frame.
  queue: Vec<FaceOffer>,
  /// The faces this frame draws, as `prepare` readied them.
  due: Vec<FaceDue>,
  /// The lights whose faces this frame draws, by their light and square.
  drawn: Vec<FaceDrawn>,
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

  /// Takes a shadowed light in view this frame, nearest first, or one out of view kept ready, after every one in view:
  /// gives it faces where it has none at the size it wants, and puts forward those of its faces wanting a draw. One
  /// kept ready keeps the faces it has, and takes new ones only from free room.
  pub fn ask(&mut self, ask: &LightShadowAsk<'_>) {
    let LightShadowAsk {
      light,
      basis,
      eye,
      forward,
      ..
    } = *ask;
    let is_spot: bool = matches!(light.kind, LightKind::Spot);
    let spatial: Vec4 = basis.get_spatial_sphere(light);
    let distance: f32 = (eye.distance(spatial.truncate()) - spatial.w).max(0.0);
    let duel: f32 = if is_spot {
      1.0 - 0.5 * forward.dot(basis.direction)
    } else {
      1.0
    };
    let cone: f32 = if is_spot { light.cone } else { LIGHT_SHADOW_POINT_CONE };
    let color: Vec3 = Vec3::from(light.color);
    let wanted: f32 = to_light_shadow_size(light.range, distance, LightBasis::to_light_intensity(color), duel, cone);
    let Some(asked) = self.choose_size(ask, wanted) else {
      return;
    };

    self.replace_set(ask, asked);
    self.push_candidates(ask, spatial, distance, cone);
  }

  /// The size of the faces a light asks for, from the texels it wants: the size it has while that stays near enough,
  /// else the nearest tile size; none where it asks nothing this frame, being kept ready already or asked for already
  /// by a nearer view.
  fn choose_size(&mut self, ask: &LightShadowAsk<'_>, wanted: f32) -> Option<u32> {
    let frame: u64 = self.frame;
    let entry: &mut LightShadowEntry = self.entries.entry(ask.handle).or_default();

    // Kept ready out of view once a frame, and never over a view's ask.
    if !ask.is_in_view {
      if entry.seen == frame || entry.warmed == frame {
        return None;
      }

      entry.warmed = frame;
    }

    let is_asked: bool = ask.is_in_view && entry.seen == frame;
    let current: Option<u32> = entry.next.as_ref().or(entry.shown.as_ref()).map(|set| set.size);
    let asked: u32 = match current {
      Some(size) if !ask.is_in_view || (wanted <= size as f32 * GROW && wanted >= size as f32 * SHRINK) => size,
      _ => to_light_shadow_tile_size(wanted),
    };

    // Another view of the scene asked for it this frame already: the size the nearer one wants stands.
    if is_asked && current.is_some_and(|size| asked <= size) {
      return None;
    }

    if ask.is_in_view {
      entry.seen = frame;
    }

    Some(asked)
  }

  /// Gives a light faces of the `asked` size: back at the size it shows while another was being drawn, the shown faces
  /// stand and the other goes; at a size it neither shows nor draws, new faces replace those it draws.
  fn replace_set(&mut self, ask: &LightShadowAsk<'_>, asked: u32) {
    // Out of the map while the faces are made, which may evict another light's.
    let Some(mut entry) = self.entries.remove(&ask.handle) else {
      return;
    };

    if entry.shown.as_ref().is_some_and(|set| set.size == asked) {
      if let Some(next) = entry.next.take() {
        Self::release_set(&mut self.allocator, next);
      }
    } else if entry.next.as_ref().is_none_or(|set| set.size != asked) {
      if let Some(next) = entry.next.take() {
        Self::release_set(&mut self.allocator, next);
      }

      entry.next = self.create_set(ask, asked);
    }

    self.entries.insert(ask.handle, entry);
  }

  /// Puts forward the faces of a light wanting a draw: those holding nothing yet or changed under, those its light
  /// carried, and those the sway or a drawable kind's casters move in: the light's bounding sphere (`spatial`),
  /// `distance` metres from the eye, its cone as `cone`.
  fn push_candidates(&mut self, ask: &LightShadowAsk<'_>, spatial: Vec4, distance: f32, cone: f32) {
    let LightShadowAsk {
      handle,
      light,
      basis,
      eye,
      is_in_view,
      is_scene_first,
      statics,
      sway,
      casters,
      ..
    } = *ask;
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
      .swaying_contents
      .is_none_or(|at| statics.is_changed_since(at, touches))
    {
      entry.swaying_contents = Some(contents);
      entry.swaying = Self::measure_swaying(basis.position, spatial, near, sway.places);
    }

    // A kind's casters stand about the eye: a light near it whose reach meets them casts their shadow. The scene's first
    // view draws the faces with its own casters, so only its asks say which lights cast them; another view's leave that
    // be.
    if is_scene_first {
      let is_cast = |casters: &&LightCasters| {
        distance < casters.light_distance && eye.distance(spatial.truncate()) < spatial.w + casters.reach
      };

      entry.casting = casters.iter().filter(is_cast).map(|casters| casters.kind).collect();
      entry.casting_lean = casters
        .iter()
        .filter(is_cast)
        .map(|casters| casters.lean)
        .fold(0.0, f32::max);
    }

    let casting: DrawableKinds = entry.casting;
    let swaying: f32 = entry.swaying.max(entry.casting_lean / near);

    self.is_swaying |= sway.amplitude * swaying > 0.0;
    // How wide a face's texels are, for each metre from the light, times the square's side.
    let spread: f32 = 2.0 * ((cone + LIGHT_SHADOW_WIDENING) * 0.5).tan();

    for (set_kind, set) in [(FaceSet::Shown, &mut entry.shown), (FaceSet::Next, &mut entry.next)] {
      if let Some(set) = set {
        for (face, state) in set.faces.iter_mut().enumerate() {
          let key: FaceKey = FaceKey {
            handle,
            set: set_kind,
            face,
          };
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

          let candidate = |is_current: bool, by: f32, wait_most: u32| FaceCandidate {
            order: FaceOrder {
              is_away,
              is_current,
              by,
            },
            key,
            wait_most,
          };

          if state.drawn.is_none_or(|drawn| statics.is_changed_since(drawn, touches)) || state.casting != casting {
            state.drawn_static = None;
            self.candidates.push(candidate(false, distance, stale_wait));
          } else if is_moved {
            self.candidates.push(candidate(true, state.drawn_at, moved_wait));
          } else if sway.is_redrawn(swaying, spread / state.tile.size as f32, state.drawn_at) {
            self.candidates.push(candidate(true, state.drawn_at, SWAY_WAIT_MOST));
          }
        }
      }
    }
  }

  /// Offers `scheduler` the faces wanting a draw this frame, the nearest lights' first, each once however many views
  /// asked for it, by their texels: one holding nothing yet or changed under within `STALE_WAIT_MOST` frames, one
  /// swaying within `SWAY_WAIT_MOST`, one its light carried at once.
  pub fn finish(&mut self, scheduler: &mut PeriodicScheduler) {
    self.candidates.sort_by(|a, b| {
      (a.order.is_away, a.order.is_current)
        .cmp(&(b.order.is_away, b.order.is_current))
        .then(a.order.by.total_cmp(&b.order.by))
    });
    let mut queued: HashSet<FaceKey> = HashSet::new();
    let faces: Vec<FaceCandidate> = self
      .candidates
      .iter()
      .filter(|candidate| queued.insert(candidate.key))
      .take(FACE_BUDGET)
      .copied()
      .collect();

    self.queue.clear();

    for FaceCandidate { key, wait_most, .. } in faces {
      let Some(state) = self.get_face(key) else {
        continue;
      };
      let texels: f32 = (state.tile.size as f32).powi(2);
      let job: PeriodicJob = if wait_most == 0 {
        PeriodicJob::new_now(&FACE_PASSES, FACE_SEED, texels)
      } else {
        PeriodicJob::new_waiting(&FACE_PASSES, FACE_SEED, texels, state.waited, wait_most)
      };

      self.queue.push(FaceOffer {
        key,
        ticket: scheduler.offer(job),
      });
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

    for FaceOffer { key, ticket } in std::mem::take(&mut self.queue) {
      let slot: usize = self.due.len();
      let is_granted: bool = scheduler.is_granted(ticket);
      let Some(entry) = self.entries.get_mut(&key.handle) else {
        continue;
      };
      // What sways in its reach, trees or a kind's leaning casters, splits it into what stands still, kept in its static
      // tile, and what moves over its copy: drawing it whole again for the grass alone redrew every caster.
      let is_split: bool = self.statics.is_some() && (entry.swaying > 0.0 || entry.casting_lean > 0.0);
      let casting: DrawableKinds = entry.casting;
      let set: Option<&mut LightShadowSet> = match key.set {
        FaceSet::Shown => entry.shown.as_mut(),
        FaceSet::Next => entry.next.as_mut(),
      };
      let Some(state) = set.and_then(|set| set.faces.get_mut(key.face)) else {
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
      state.casting = casting;

      if draw == FaceDraw::Static {
        state.drawn_static = Some(contents);
      }

      self.due.push(FaceDue {
        slot,
        tile: state.tile,
        draw,
        casting,
      });
      self.drawn.push(FaceDrawn {
        handle: key.handle,
        tile: state.tile,
      });
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
  /// every face culled into its slot in one compute pass, then every face drawn into its square in one render pass, what
  /// the drawable `kinds` cast over those whose light casts from them.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes,
      textures,
      statics,
      cull: params,
      ..
    }: ViewRecord<'a>,
    kinds: &[DrawableKindRef<'a>],
  ) {
    if self.due.is_empty() {
      return;
    }

    let scene: &StaticSceneHandles = &statics;
    let template: GraphBuffer = scene.args_template;
    let args: GraphBuffer = bindings.import_buffer(graph, "light shadow draw arguments", &self.args);
    let lists: GraphBuffer = bindings.import_buffer(graph, "light shadow lists", self.lists.get_buffer());
    let Some(atlas) = &self.atlas else {
      return;
    };
    let atlas: GraphTexture = bindings.import_view(graph, "light shadow atlas", atlas);
    // The kinds this frame's record brought, which a face draws only those of.
    let present: DrawableKinds = kinds.iter().map(DrawableKindRef::get_kind).collect();
    // Each slot's cull into its range of the lists and the arguments, and its draws of that range.
    let slots: Vec<FaceSlot> = self
      .due
      .iter()
      .map(|due| {
        let (list_offset, args_offset): (u64, u64) =
          (due.slot as u64 * self.region, due.slot as u64 * self.args_stride);

        FaceSlot {
          due: FaceDue {
            casting: due.casting.intersect(present),
            ..*due
          },
          cull: scene.get_cull_parameters(
            StorageArrayMut::new_range(lists, list_offset, self.region),
            StorageArrayMut::new_range(args, args_offset, StaticCullScratch::ARGS_SIZE),
          ),
          layouts: scene.get_layout_draws(StorageArray::new_range(lists, list_offset, self.region)),
        }
      })
      .collect();

    graph
      .add_encoder_pass(FACE_PASSES[0])
      .buffer(template, GraphBufferAccess::CopySource)
      .buffer(args, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let (template, args) = (context.get_buffer(template), context.get_buffer(args));

        for due in &self.due {
          context.get_encoder().copy_buffer_to_buffer(
            template,
            0,
            args,
            due.slot as u64 * self.args_stride,
            StaticCullScratch::ARGS_SIZE,
          );
        }
      });
    slots
      .iter()
      .fold(graph.add_compute_pass(FACE_PASSES[1]), |builder, slot| {
        builder.parameters(&slot.cull)
      })
      .record({
        let culls: Vec<FaceSlot> = slots.clone();

        move |context| {
          for FaceSlot { due, cull, .. } in &culls {
            passes
              .scene
              .cull
              .record_shadow(context, &self.views[due.slot], cull, params, ShadowCull::LightFace);
          }
        }
      });
    // What stands still of the faces whose static tiles are stale, into those tiles, before any tile is restored.
    let restore: Option<ShadowTileRestoreParameters> =
      self.statics.as_ref().map(|statics| ShadowTileRestoreParameters {
        static_atlas: bindings.import_view(&mut *graph, "light shadow static atlas", statics),
      });
    let statics: Vec<FaceSlot> = slots
      .iter()
      .filter(|slot| slot.due.draw == FaceDraw::Static)
      .copied()
      .collect();

    if let (Some(restore), false) = (restore, statics.is_empty()) {
      statics
        .iter()
        .fold(
          graph
            .add_raster_pass(FACE_PASSES[3])
            .depth(GraphDepthAttachment::new(restore.static_atlas, wgpu::LoadOp::Load))
            .buffer(args, GraphBufferAccess::Indirect),
          |builder, slot| StaticDraws::declare_layouts(builder, &slot.layouts),
        )
        .record(move |context| {
          let args: &wgpu::Buffer = context.get_buffer(args);

          for FaceSlot { due, layouts, .. } in &statics {
            passes.lighting.shadow.record_tile(
              context,
              due.tile.get_region(),
              None,
              StaticLayoutBindings::new(&self.views[due.slot], layouts, textures),
              IndirectArgs {
                buffer: args,
                offset: due.slot as u64 * self.args_stride,
              },
              ShadowCasters::Fixed,
            );
          }
        });
    }

    let is_restored: bool = slots.iter().any(|slot| slot.due.draw != FaceDraw::Whole);
    // The kinds any face this frame draws casters of, which the pass reads.
    let drawn: DrawableKinds = slots
      .iter()
      .fold(DrawableKinds::default(), |drawn, slot| drawn.union(slot.due.casting));
    let drawing: Vec<DrawableKindRef<'a>> = kinds
      .iter()
      .filter(|kind| drawn.has(kind.get_kind()))
      .copied()
      .collect();
    let builder = slots.iter().fold(
      graph
        .add_raster_pass(FACE_PASSES[2])
        .depth(GraphDepthAttachment::new(atlas, wgpu::LoadOp::Load))
        .buffer(args, GraphBufferAccess::Indirect),
      |builder, slot| StaticDraws::declare_layouts(builder, &slot.layouts),
    );
    let builder = match restore.filter(|_| is_restored) {
      Some(restore) => builder.parameters(&restore),
      None => builder,
    };
    let builder = drawing.iter().fold(builder, |builder, kind| kind.declare(builder));

    builder.record(move |context| {
      let args: &wgpu::Buffer = context.get_buffer(args);

      for FaceSlot { due, layouts, .. } in &slots {
        // A swaying light's face over what stands still, as kept; any other drawn whole.
        let (restored, casters): (Option<&ShadowTileRestoreParameters>, ShadowCasters) = match (due.draw, &restore) {
          (FaceDraw::Static | FaceDraw::Swaying, Some(restore)) => (Some(restore), ShadowCasters::Swaying),
          _ => (None, ShadowCasters::All),
        };

        passes.lighting.shadow.record_tile(
          context,
          due.tile.get_region(),
          restored,
          StaticLayoutBindings::new(&self.views[due.slot], layouts, textures),
          IndirectArgs {
            buffer: args,
            offset: due.slot as u64 * self.args_stride,
          },
          casters,
        );

        for kind in drawing.iter().filter(|kind| due.casting.has(kind.get_kind())) {
          kind.record_shadow(context, &self.views[due.slot]);
        }
      }
    });
  }

  /// Forgets that the faces this frame drew were drawn, where its frame was not submitted: each draws again, and its
  /// light waits for it.
  pub fn abandon_frame(&mut self) {
    for FaceDrawn { handle, tile } in std::mem::take(&mut self.drawn) {
      let Some(entry) = self.entries.get_mut(&handle) else {
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

  /// Lets every light's faces go, for lights that replace them: each light asked for after takes faces anew.
  pub fn forget_lights(&mut self) {
    self.entries.clear();
    self.drawn.clear();
    self.due.clear();
    self.allocator = ShadowTileAllocator::new(Self::ATLAS_SIZE, LIGHT_SHADOW_MIN_SIZE);
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
  fn get_face(&self, FaceKey { handle, set, face }: FaceKey) -> Option<&LightShadowFace> {
    let entry: &LightShadowEntry = self.entries.get(&handle)?;
    let set: &LightShadowSet = match set {
      FaceSet::Shown => entry.shown.as_ref(),
      FaceSet::Next => entry.next.as_ref(),
    }?;

    set.faces.get(face)
  }

  /// The faces a light asks for (`ask`) at a size, room made from the lights out of view longest where a view sees it,
  /// and at smaller squares where there is still none.
  fn create_set(&mut self, ask: &LightShadowAsk<'_>, size: u32) -> Option<LightShadowSet> {
    let LightShadowAsk {
      light,
      basis,
      is_in_view: may_evict,
      ..
    } = *ask;
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
            casting: DrawableKinds::default(),
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
