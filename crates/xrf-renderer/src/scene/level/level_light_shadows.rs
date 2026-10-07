use std::collections::HashMap;

use glam::{Vec3, Vec4};
use xrf_math::EPS_S;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, ProxyHandle,
};
use xrf_visual::{LightDescription, LightKind};

use crate::camera::camera_view::CameraView;
use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_rect::RenderRect;
use crate::lighting::light_basis::{LightBasis, to_light_intensity};
use crate::lighting::light_shadow_size::{
  LIGHT_SHADOW_MIN_SIZE, LIGHT_SHADOW_POINT_CONE, LIGHT_SHADOW_POINT_FACES, LIGHT_SHADOW_WIDENING,
  to_light_shadow_size, to_light_shadow_tile_size,
};
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::light_shadow_entry::LightShadowEntry;
use crate::scene::level::light_shadow_face::LightShadowFace;
use crate::scene::level::light_shadow_set::LightShadowSet;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::level::shadow_tile::ShadowTile;
use crate::scene::level::shadow_tile_allocator::ShadowTileAllocator;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;

/// Texels the light shadow atlas is across.
pub const LIGHT_SHADOW_ATLAS_SIZE: u32 = 4096;

/// Shadow faces drawn at most in one frame, so a level opening fills the atlas over a few frames rather than in one.
const FACE_BUDGET: usize = 8;

/// How far a light's wanted size may stray from the square it was asked at before it is asked at another.
const GROW: f32 = 1.5;
const SHRINK: f32 = 0.66;

/// Where a face's projection starts where the light gives none: `light::virtual_size`'s default.
const DEFAULT_NEAR: f32 = 0.1;

/// A face wanting a draw: what orders it (stale, then how near or how long ago drawn), its light, whether of the light's
/// next set, and which face.
type FaceCandidate = ((bool, f32), ProxyHandle<LightDescription>, bool, usize);

/// Each slot's bind groups, with what they were made from.
type SlotGroups<K, G> = Option<(K, Vec<G>)>;

/// The local lights' shadows: a square of an atlas a face, sized as the engine sizes its maps, drawn once and kept
/// while the scene it casts from stays, a few faces a frame, the nearest lights first; a face over swaying trees is
/// drawn again once a sway interval, where the lean shows at its texels. A light lights only once its
/// faces are drawn; one asked at another size keeps its old faces until its new ones are. Room is made from the lights
/// out of view longest, and where there is still none, a light is asked at smaller squares. Each face drawn in a frame
/// has a slot of its own in the lists and the draw arguments, so every face is culled before any is drawn, and all are
/// drawn in one render pass.
pub struct LevelLightShadows {
  atlas: wgpu::TextureView,
  allocator: ShadowTileAllocator,
  entries: HashMap<ProxyHandle<LightDescription>, LightShadowEntry>,
  frame: u64,
  /// The faces wanting a draw this frame, as their light, set and face, behind what orders them: stale ones first,
  /// nearest lights first, then the ones over what sways, drawn longest ago first.
  candidates: Vec<FaceCandidate>,
  /// The faces drawn this frame, a camera each from the pool.
  queue: Vec<(ProxyHandle<LightDescription>, bool, usize)>,
  /// The faces this frame draws, by their slot and their square, as `prepare` readied them.
  due: Vec<(usize, ShadowTile)>,
  /// A face's camera, a slot each.
  views: Vec<ViewBinding>,
  /// Every slot's visible list, `region` bytes apart.
  lists: GrowableBuffer,
  region: u64,
  /// Every slot's draw arguments, `args_stride` bytes apart, each `args_size` long.
  args: wgpu::Buffer,
  args_size: u64,
  args_stride: u64,
  /// What a slot's range of a buffer starts at a multiple of.
  alignment: u64,
  /// Each slot's cull and draw bind groups, with the scene generation, the lists' generation and region, and the
  /// targets' epoch they bind.
  cull_groups: SlotGroups<(u64, u64, u64, u64), wgpu::BindGroup>,
  draw_groups: SlotGroups<(u64, u64, u64), [wgpu::BindGroup; StaticLayout::COUNT]>,
}

impl LevelLightShadows {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64) -> Self {
    let alignment: u64 = u64::from(device.limits().min_storage_buffer_offset_alignment);
    let args_stride: u64 = args_size.next_multiple_of(alignment);
    let atlas: wgpu::TextureView = device
      .create_texture(&wgpu::TextureDescriptor {
        label: Some("light shadow atlas"),
        size: wgpu::Extent3d {
          width: LIGHT_SHADOW_ATLAS_SIZE,
          height: LIGHT_SHADOW_ATLAS_SIZE,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
      .create_view(&Default::default());

    Self {
      atlas,
      allocator: ShadowTileAllocator::new(LIGHT_SHADOW_ATLAS_SIZE, LIGHT_SHADOW_MIN_SIZE),
      entries: HashMap::new(),
      frame: 0,
      candidates: Vec::new(),
      queue: Vec::new(),
      due: Vec::new(),
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
      args_size,
      args_stride,
      alignment,
      cull_groups: None,
      draw_groups: None,
    }
  }

  /// Texels of the atlas the lights' faces hold, of its whole.
  pub fn get_atlas_use(&self) -> RenderPoolUse {
    RenderPoolUse {
      used: self.allocator.get_used().min(u64::from(u32::MAX)) as u32,
      capacity: LIGHT_SHADOW_ATLAS_SIZE * LIGHT_SHADOW_ATLAS_SIZE,
    }
  }

  pub fn get_atlas(&self) -> &wgpu::TextureView {
    &self.atlas
  }

  /// Starts a frame's asks.
  pub fn begin(&mut self) {
    self.frame += 1;
    self.candidates.clear();
    self.queue.clear();
  }

  /// Takes a shadowed light in view this frame, nearest first: gives it faces where it has none at the size it wants,
  /// and puts forward those of its faces wanting a draw.
  #[allow(clippy::too_many_arguments)]
  pub fn ask(
    &mut self,
    handle: ProxyHandle<LightDescription>,
    light: &LightDescription,
    basis: &LightBasis,
    eye: Vec3,
    forward: Vec3,
    color: Vec3,
    contents: usize,
    sway: &ShadowSway<'_>,
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
    let wanted: f32 = to_light_shadow_size(light.range, distance, to_light_intensity(color), duel, cone);
    let entry: &mut LightShadowEntry = self.entries.entry(handle).or_default();

    entry.seen = self.frame;

    let current: Option<u32> = entry.next.as_ref().or(entry.shown.as_ref()).map(|set| set.size);
    let asked: u32 = match current {
      Some(size) if wanted <= size as f32 * GROW && wanted >= size as f32 * SHRINK => size,
      _ => to_light_shadow_tile_size(wanted),
    };

    // Back at the size it shows while another was being drawn: the shown faces stand, and the other goes.
    if entry.shown.as_ref().is_some_and(|set| set.size == asked) {
      if let Some(next) = entry.next.take() {
        release_set(&mut self.allocator, next);
      }
    } else if entry.next.as_ref().is_none_or(|set| set.size != asked) {
      if let Some(next) = self.entries.get_mut(&handle).and_then(|entry| entry.next.take()) {
        release_set(&mut self.allocator, next);
      }

      let set: Option<LightShadowSet> = self.create_set(light, basis, asked);

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

    if entry.swaying.is_none_or(|(at, _)| at != contents) {
      entry.swaying = Some((contents, measure_swaying(basis.position, spatial, near, sway.places)));
    }

    let swaying: f32 = entry.swaying.map_or(0.0, |(_, swaying)| swaying);
    // How wide a face's texels are, for each metre from the light, times the square's side.
    let spread: f32 = 2.0 * ((cone + LIGHT_SHADOW_WIDENING) * 0.5).tan();

    for (is_next, set) in [(false, &mut entry.shown), (true, &mut entry.next)] {
      if let Some(set) = set {
        for (face, state) in set.faces.iter_mut().enumerate() {
          // A light a motion carries has moved off where its faces look from: each looks from where it stands now,
          // and is drawn again as a swaying one is, lighting with what it holds meanwhile.
          let is_moved: bool = state.view.position != basis.position;

          if is_moved {
            state.view = to_face_view(light, basis, face, set.near, set.far);
          }

          if state.drawn != Some(contents) {
            self.candidates.push(((false, distance), handle, is_next, face));
          } else if is_moved || sway.is_redrawn(swaying, spread / state.tile.size as f32, state.drawn_at) {
            self.candidates.push(((true, state.drawn_at), handle, is_next, face));
          }
        }
      }
    }
  }

  /// Queues the faces drawn this frame, the nearest lights' first, and takes a light's new faces in place of its old
  /// ones once all are drawn.
  pub fn finish(&mut self) {
    self
      .candidates
      .sort_by(|a, b| a.0.0.cmp(&b.0.0).then(a.0.1.total_cmp(&b.0.1)));
    self.queue = self
      .candidates
      .iter()
      .take(FACE_BUDGET)
      .map(|(_, index, is_next, face)| (*index, *is_next, *face))
      .collect();
  }

  /// The faces a light lights with this frame, every one drawn, or none where it has none whole.
  pub fn get_set(&self, handle: ProxyHandle<LightDescription>) -> Option<&LightShadowSet> {
    self.entries.get(&handle)?.shown.as_ref().filter(|set| set.is_drawn())
  }

  /// Readies the faces queued this frame to be culled into their slots and drawn into their squares of the atlas, and
  /// lets a light's new faces take the place of its old once all are drawn; `encoder` takes the copies a list growing
  /// makes.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    passes: LevelPasses<'_>,
    frame: &ShadowFrame<'_>,
  ) {
    let scene = frame.scene;
    let contents: usize = scene.get_contents();

    self.due.clear();

    if self.queue.is_empty() {
      return;
    }

    self.region = ((scene.get_list_capacity().max(1) as u64) * 8).next_multiple_of(self.alignment);
    self.lists.reserve(device, encoder, self.region * FACE_BUDGET as u64);

    let cull_key: (u64, u64, u64, u64) = (
      scene.get_generation(),
      self.lists.get_generation(),
      self.region,
      frame.targets_epoch,
    );

    if self.cull_groups.as_ref().is_none_or(|(key, _)| *key != cull_key) {
      let groups: Vec<wgpu::BindGroup> = (0..FACE_BUDGET)
        .map(|slot| {
          passes.cull.create_bind_group(
            device,
            scene,
            frame.cull_params,
            frame.pyramid,
            frame.occlusion,
            (self.get_list_range(slot), self.get_args_range(slot)),
          )
        })
        .collect();

      self.cull_groups = Some((cull_key, groups));
    }

    let draw_key: (u64, u64, u64) = (scene.get_generation(), self.lists.get_generation(), self.region);

    if self.draw_groups.as_ref().is_none_or(|(key, _)| *key != draw_key) {
      let groups: Vec<[wgpu::BindGroup; StaticLayout::COUNT]> = (0..FACE_BUDGET)
        .map(|slot| {
          passes
            .gbuffer
            .create_layout_groups(device, scene, self.get_list_range(slot))
        })
        .collect();

      self.draw_groups = Some((draw_key, groups));
    }

    for (slot, (index, is_next, face)) in self.queue.iter().enumerate() {
      let Some(state) = self
        .entries
        .get_mut(index)
        .and_then(|entry| {
          if *is_next {
            entry.next.as_mut()
          } else {
            entry.shown.as_mut()
          }
        })
        .and_then(|set| set.faces.get_mut(*face))
      else {
        continue;
      };
      let view: &ViewBinding = &self.views[slot];

      view.write(
        queue,
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
      self.due.push((slot, state.tile));
      state.drawn = Some(contents);
      state.drawn_at = frame.sway.time;
    }

    // A light's new faces, all drawn, take the place of its old.
    for entry in self.entries.values_mut() {
      if entry.next.as_ref().is_some_and(LightShadowSet::is_drawn) {
        if let Some(shown) = entry.shown.take() {
          release_set(&mut self.allocator, shown);
        }

        entry.shown = entry.next.take();
      }
    }
  }

  /// Declares the faces this frame's `prepare` readied: each slot's arguments started from those a cull starts with,
  /// every face culled into its slot in one compute pass, then every face drawn into its square in one render pass.
  pub fn add_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    passes: LevelPasses<'a>,
    (scene, params, textures): (&'a StaticScene, &'a StaticCullParams, &'a wgpu::BindGroup),
  ) {
    let (Some((_, cull_groups)), Some((_, draw_groups))) = (&self.cull_groups, &self.draw_groups) else {
      return;
    };

    if self.due.is_empty() {
      return;
    }

    let template: GraphBuffer = bindings.import_buffer(graph, "static draw arguments", &scene.args_template);
    let args: GraphBuffer = bindings.import_buffer(graph, "light shadow draw arguments", &self.args);
    let lists: GraphBuffer = bindings.import_buffer(graph, "light shadow lists", self.lists.get_buffer());
    let atlas: GraphTexture = bindings.import_view(graph, "light shadow atlas", &self.atlas);

    graph
      .add_encoder_pass("light shadow arguments")
      .buffer(template, GraphBufferAccess::CopySource)
      .buffer(args, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let (template, args) = (context.get_buffer(template), context.get_buffer(args));

        for (slot, _) in &self.due {
          context.get_encoder().copy_buffer_to_buffer(
            template,
            0,
            args,
            *slot as u64 * self.args_stride,
            self.args_size,
          );
        }
      });
    graph
      .add_compute_pass("light shadow cull")
      .buffer(args, GraphBufferAccess::StorageReadWrite)
      .buffer(lists, GraphBufferAccess::StorageWrite)
      .record(move |context| {
        for (slot, _) in &self.due {
          passes.cull.record_shadow(
            context.get_pass(),
            &self.views[*slot],
            &cull_groups[*slot],
            params,
            true,
          );
        }
      });
    graph
      .add_raster_pass("light shadows")
      .depth(GraphDepthAttachment::new(atlas, wgpu::LoadOp::Load))
      .buffer(args, GraphBufferAccess::Indirect)
      .buffer(lists, GraphBufferAccess::StorageRead)
      .record(move |context| {
        let args: &wgpu::Buffer = context.get_buffer(args);

        for (slot, tile) in &self.due {
          passes.shadow.record_tile(
            context.get_pass(),
            *tile,
            (&self.views[*slot], &draw_groups[*slot], textures),
            (args, *slot as u64 * self.args_stride),
          );
        }
      });
  }

  /// A slot's range of the lists.
  fn get_list_range(&self, slot: usize) -> wgpu::BufferBinding<'_> {
    wgpu::BufferBinding {
      buffer: self.lists.get_buffer(),
      offset: slot as u64 * self.region,
      size: wgpu::BufferSize::new(self.region),
    }
  }

  /// A slot's range of the draw arguments.
  fn get_args_range(&self, slot: usize) -> wgpu::BufferBinding<'_> {
    wgpu::BufferBinding {
      buffer: &self.args,
      offset: slot as u64 * self.args_stride,
      size: wgpu::BufferSize::new(self.args_size),
    }
  }

  /// A light's faces at a size, room made from the lights out of view longest, and at smaller squares where there is
  /// still none.
  fn create_set(&mut self, light: &LightDescription, basis: &LightBasis, size: u32) -> Option<LightShadowSet> {
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
            view: to_face_view(light, basis, face, near, far),
            drawn: None,
            drawn_at: 0.0,
          })
          .collect();

        return Some(LightShadowSet { size, near, far, faces });
      }

      if !self.evict_oldest() {
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
        .for_each(|set| release_set(&mut self.allocator, set));
    }

    true
  }
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
