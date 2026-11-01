use std::collections::HashMap;

use xrf_renderer_core::ProxyHandle;
use xrf_visual::LightDescription;

use crate::contract::report::frame::render_pool_use::RenderPoolUse;
use crate::feature::scene::drawable::drawable_kinds::DrawableKinds;
use crate::feature::scene::statics::cull::static_cull_parameters::StaticCullParameters;
use crate::feature::scene::statics::draw::static_draw_parameters::StaticDrawParameters;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
use crate::lighting::lamp::light_shadow_size::LIGHT_SHADOW_MIN_SIZE;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::lights::atlas::light_shadow_atlas::LightShadowAtlas;
use crate::scene::level::lights::atlas::light_shadow_entry::LightShadowEntry;
use crate::scene::level::lights::atlas::light_shadow_set::LightShadowSet;
use crate::scene::level::lights::atlas::shadow_tile::ShadowTile;
use crate::scene::level::lights::atlas::shadow_tile_allocator::ShadowTileAllocator;
use crate::scene::static_scene::draw::static_cull_scratch::StaticCullScratch;
use crate::scene::static_scene::geometry::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;

mod level_light_shadows_asks;
mod level_light_shadows_faces;
mod level_light_shadows_record;

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

  /// The faces a light lights with this frame, every one drawn, or none where it has none whole.
  pub fn get_set(&self, handle: ProxyHandle<LightDescription>) -> Option<&LightShadowSet> {
    self.entries.get(&handle)?.shown.as_ref().filter(|set| set.is_drawn())
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
}
