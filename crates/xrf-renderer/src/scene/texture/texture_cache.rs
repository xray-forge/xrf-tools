use std::borrow::Borrow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::contract::report::texture::render_texture_group::RenderTextureGroup;
use crate::contract::report::texture::render_texture_report::RenderTextureReport;
use crate::contract::report::texture::render_texture_state::RenderTextureState;
use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::texture::decoded_texture::DecodedTexture;
use crate::scene::texture::texture_role::TextureRole;
use crate::thread::queue::loader_receiver::LoaderReceiver;
use crate::thread::queue::render_workers::RenderWorkers;

mod texture_cache_binding;
mod texture_cache_loads;

/// Texels a side of the checker standing in for a base texture that cannot be had, and of each of its squares.
const CHECKER_SIZE: u32 = 16;
const CHECKER_SQUARE: u32 = 2;

/// Bytes of texture uploaded at most each frame while a level opens, so an open spreads its uploads rather than
/// stalling a frame.
const UPLOAD_BYTES: u64 = 64 * 1024 * 1024;

/// Textures offered at most a frame once every level stands; milliseconds a byte of one costs to upload, and frames
/// one waits at most.
const UPLOADS_OFFERED: usize = 16;
const UPLOAD_SEED: f32 = 3e-8;
const UPLOAD_WAIT_MOST: u32 = 4;

/// What uploading a texture is named as a job: nothing the timer measures, so its cost stays its seed.
static UPLOAD_PASSES: [&str; 1] = ["texture upload"];

/// Slots a bindless array holds at most, below what any adapter meeting the baseline offers.
const MAX_SLOTS: u32 = 16_384;

/// What a texture load came to, sent from a loader thread: its slot, the slot's generation it was asked under, and the
/// texture.
type TextureLoad = (u32, u32, Result<Option<DecodedTexture>, String>, Option<u64>);

/// Every texture the renderer's scenes sample, once each by scope and reference, in one bindless array: a material
/// names its texture by slot, so draws are never split by texture. A slot no scene samples any more is freed and given
/// to the next texture asked for.
pub struct TextureCache {
  layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  /// The sampler's anisotropy clamp, one for trilinear.
  anisotropy: u16,
  bind_group: wgpu::BindGroup,
  /// Each slot's view; a slot not loaded holds the missing texture's.
  views: Vec<wgpu::TextureView>,
  states: Vec<RenderTextureState>,
  /// Each slot's reference, empty for the missing texture's.
  references: Vec<String>,
  /// What each slot is sampled as, which decides what stands in for it.
  roles: Vec<TextureRole>,
  /// Whether each slot's texture is a distortion map uploaded without any blue (`is_blue_free`).
  blue_free: Vec<bool>,
  /// What stands in for a texture of each role while it loads, by `TextureRole as usize`.
  neutral: Vec<wgpu::TextureView>,
  /// What stands in for a base texture that cannot be had.
  checker: wgpu::TextureView,
  /// Each texture's slot by the scope its source resolves it in, then its reference, so a lookup borrows both.
  by_reference: HashMap<String, HashMap<String, u32>>,
  /// Each slot's key in `by_reference`, `None` for the missing texture's and for a freed slot.
  keys: Vec<Option<(String, String)>>,
  /// Bumped each time a slot is freed or loaded again, so a load asked for its last texture is not uploaded into its
  /// next.
  generations: Vec<u32>,
  /// Where each slot's texture is read from, `None` for the missing texture's and for a freed slot.
  sources: Vec<Option<Arc<dyn RenderAssetSource>>>,
  /// `texture_lod`: how much coarser than stored the textures load, the engine's rule deciding each one's levels.
  lod: u32,
  /// Whether a terrain's heights are loaded: held deferred, their neutral standing in, until a view first reads them.
  is_heights_wanted: bool,
  /// The stamp of the file each slot was last read from, none where it has none.
  stamps: Vec<Option<u64>>,
  /// Slots freed, taken again before the array grows.
  free: Vec<u32>,
  sender: Sender<TextureLoad>,
  /// Behind a lock only so the cache is shared by the passes a frame records; read with `&mut self`, never locked.
  receiver: LoaderReceiver<TextureLoad>,
  /// Loaded, waiting for a frame's upload budget, each beside the generation it was asked under.
  uploads: VecDeque<(u32, u32, DecodedTexture)>,
  /// Frames the first upload waiting has waited.
  waited: u32,
  capacity: u32,
  is_dirty: bool,
  /// The cubes surfaces named as their environment, each slot past the first in turn, empty where freed; loaded as
  /// weather textures are.
  environments: Vec<String>,
  /// Counts every change of `environments`, so a view knows to bind them again.
  environments_generation: u64,
  workers: RenderWorkers,
}

impl TextureCache {
  /// Environment cubes the scenes' surfaces may mix toward, the first slot standing for none.
  pub const ENVIRONMENT_SLOTS: u32 = 16;

  /// The slot a surface binds where it names no texture of a kind: mid grey, never sampled by a surface without one.
  pub const MISSING_SLOT: u32 = 0;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, workers: &RenderWorkers) -> Self {
    // The environment cubes a composited draw binds beside the array share the stage's budget.
    let capacity: u32 = device
      .limits()
      .max_binding_array_elements_per_shader_stage
      .saturating_sub(Self::ENVIRONMENT_SLOTS)
      .clamp(1, MAX_SLOTS);
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("textures"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: NonZeroU32::new(capacity),
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });
    let anisotropy: u16 = 8;
    let sampler: wgpu::Sampler = Self::create_sampler(device, anisotropy);
    let missing: wgpu::TextureView = Self::create_solid(device, queue, [128, 128, 128, 255]);
    let neutral: Vec<wgpu::TextureView> = TextureRole::ALL
      .into_iter()
      .map(|role| Self::create_solid(device, queue, role.get_neutral()))
      .collect();
    let checker: wgpu::TextureView = Self::create_checker(device, queue);
    let (sender, receiver) = LoaderReceiver::channel();
    let views: Vec<wgpu::TextureView> = vec![missing];
    let bind_group: wgpu::BindGroup = Self::create_bind_group(device, &layout, &sampler, &views);

    Self {
      layout,
      sampler,
      anisotropy,
      bind_group,
      views,
      states: vec![RenderTextureState::Missing],
      references: vec![String::new()],
      roles: vec![TextureRole::Detail],
      blue_free: vec![false],
      neutral,
      checker,
      by_reference: HashMap::new(),
      keys: vec![None],
      generations: vec![0],
      sources: vec![None],
      lod: 0,
      is_heights_wanted: false,
      stamps: vec![None],
      free: Vec::new(),
      sender,
      receiver,
      uploads: VecDeque::new(),
      waited: 0,
      capacity,
      is_dirty: false,
      environments: Vec::new(),
      environments_generation: 0,
      workers: workers.clone(),
    }
  }

  pub fn get_layout(&self) -> &wgpu::BindGroupLayout {
    &self.layout
  }

  /// The bindless group, bound at group 1, below the per-batch groups, so switching batches never re-walks its views.
  pub fn get_bind_group(&self) -> &wgpu::BindGroup {
    &self.bind_group
  }

  /// The slot a texture reference samples from, loading it on a loader thread the first time its source's scope asks
  /// for it; until it loads, its role's neutral stands in.
  pub fn request(&mut self, reference: &str, role: TextureRole, source: &Arc<dyn RenderAssetSource>) -> u32 {
    if reference.is_empty() {
      return Self::MISSING_SLOT;
    }

    let scope: &str = source.get_texture_scope();

    if let Some(&slot) = self.by_reference.get(scope).and_then(|slots| slots.get(reference)) {
      // Asked again by a source opened since, as a level reopened: the file may have changed meanwhile.
      if self.sources[slot as usize]
        .as_ref()
        .is_some_and(|known| !Arc::ptr_eq(known, source))
      {
        self.sources[slot as usize] = Some(Arc::clone(source));
        self.recheck(slot);
      }

      return slot;
    }

    let slot: u32 = match self.free.pop() {
      Some(slot) => {
        let index: usize = slot as usize;

        self.views[index] = self.neutral[role as usize].clone();
        self.states[index] = RenderTextureState::Loading;
        self.references[index] = reference.to_owned();
        self.roles[index] = role;
        self.blue_free[index] = false;
        self.keys[index] = Some((scope.to_owned(), reference.to_owned()));
        self.sources[index] = Some(Arc::clone(source));
        self.stamps[index] = None;

        slot
      }
      None if (self.views.len() as u32) < self.capacity => {
        self.views.push(self.neutral[role as usize].clone());
        self.states.push(RenderTextureState::Loading);
        self.references.push(reference.to_owned());
        self.roles.push(role);
        self.blue_free.push(false);
        self.keys.push(Some((scope.to_owned(), reference.to_owned())));
        self.generations.push(0);
        self.sources.push(Some(Arc::clone(source)));
        self.stamps.push(None);

        self.views.len() as u32 - 1
      }
      None => {
        log::warn!(
          "Texture '{reference}' exceeds the {} the renderer binds at once",
          self.capacity
        );

        return Self::MISSING_SLOT;
      }
    };

    self.is_dirty = true;
    self
      .by_reference
      .entry(scope.to_owned())
      .or_default()
      .insert(reference.to_owned(), slot);

    if role == TextureRole::TerrainHeight && !self.is_heights_wanted {
      self.states[slot as usize] = RenderTextureState::Deferred;
    } else {
      self.load(slot);
    }

    slot
  }

  /// Loads the terrains' heights from now on, those held deferred among them, once a view first reads them: each costs
  /// an integration where the installation brings none, so they wait for an enhanced parallax or the heights view.
  pub fn want_heights(&mut self) {
    if self.is_heights_wanted {
      return;
    }

    self.is_heights_wanted = true;

    for slot in 1..self.views.len() {
      if self.states[slot] == RenderTextureState::Deferred {
        self.states[slot] = RenderTextureState::Loading;
        self.load(slot as u32);
      }
    }
  }

  /// Loads every texture again at `texture_lod`, where it changed: each keeps what it shows until its new levels land.
  pub fn set_lod(&mut self, lod: u32) {
    if lod == self.lod {
      return;
    }

    self.lod = lod;

    for slot in 1..self.views.len() {
      if self.keys[slot].is_some() && self.states[slot] != RenderTextureState::Deferred {
        self.generations[slot] = self.generations[slot].wrapping_add(1);
        self.load(slot as u32);
      }
    }
  }

  /// The environment slot a cube reference is sampled from, `0` for none or for one past the slots there are.
  pub fn request_environment(&mut self, reference: &str) -> u32 {
    if reference.is_empty() {
      return 0;
    }

    if let Some(index) = self.environments.iter().position(|it| it == reference) {
      return index as u32 + 1;
    }

    let index: usize = match self.environments.iter().position(String::is_empty) {
      Some(index) => index,
      None if self.environments.len() as u32 + 1 < Self::ENVIRONMENT_SLOTS => {
        self.environments.push(String::new());
        self.environments.len() - 1
      }
      None => {
        log::warn!(
          "Environment '{reference}' exceeds the {} the renderer binds at once",
          Self::ENVIRONMENT_SLOTS
        );

        return 0;
      }
    };

    self.environments[index] = reference.to_owned();
    self.environments_generation += 1;

    index as u32 + 1
  }

  /// The cubes asked for as environments, by slot from the second; a freed slot is empty.
  pub fn list_environments(&self) -> &[String] {
    &self.environments
  }

  /// The cube an environment slot is sampled from, or `None` for the first slot or a freed one.
  pub fn get_environment(&self, slot: u32) -> Option<&str> {
    let reference: &String = self.environments.get((slot as usize).checked_sub(1)?)?;

    (!reference.is_empty()).then_some(reference.as_str())
  }

  pub fn get_environments_generation(&self) -> u64 {
    self.environments_generation
  }

  /// Frees every environment slot no scene samples, so the next cubes asked for take their places.
  pub fn retain_environments(&mut self, sampled: &HashSet<u32>) {
    for (index, reference) in self.environments.iter_mut().enumerate() {
      if !reference.is_empty() && !sampled.contains(&(index as u32 + 1)) {
        reference.clear();
        self.environments_generation += 1;
      }
    }
  }

  /// Whether every one of some slots is uploaded or given up on.
  pub fn is_settled(&self, slots: impl IntoIterator<Item = impl Borrow<u32>>) -> bool {
    slots.into_iter().all(|slot| {
      !matches!(
        self.states.get(*slot.borrow() as usize),
        Some(RenderTextureState::Loading)
      )
    })
  }

  /// Whether a slot holds a distortion map uploaded without any blue, which never blurs the frame where it is drawn into
  /// the distortion target; not while it loads, stands in, or was asked for as anything else.
  pub fn is_blue_free(&self, slot: u32) -> bool {
    self.blue_free.get(slot as usize).copied().unwrap_or(false)
  }

  /// Of some slots, how many are uploaded or given up on.
  pub fn count_settled(&self, slots: impl IntoIterator<Item = impl Borrow<u32>>) -> u32 {
    slots
      .into_iter()
      .filter(|slot| {
        !matches!(
          self.states.get(*slot.borrow() as usize),
          Some(RenderTextureState::Loading)
        )
      })
      .count() as u32
  }

  /// What became of some slots' textures.
  pub fn describe<'a>(
    &self,
    slots: impl IntoIterator<Item = &'a u32>,
    group: RenderTextureGroup,
  ) -> Vec<RenderTextureReport> {
    slots
      .into_iter()
      .filter_map(|slot| {
        Some(RenderTextureReport {
          reference: self.references.get(*slot as usize)?.clone(),
          state: self.states.get(*slot as usize)?.clone(),
          group,
        })
      })
      .collect()
  }

  /// Frees every slot but the missing texture's and those some scene still samples, so the next textures asked for
  /// take their places; a load still on its way to a freed slot is dropped when it lands.
  pub fn retain(&mut self, sampled: &HashSet<u32>) {
    for slot in 1..self.views.len() {
      if self.keys[slot].is_none() || sampled.contains(&(slot as u32)) {
        continue;
      }

      if let Some((scope, reference)) = self.keys[slot].take()
        && let Some(slots) = self.by_reference.get_mut(&scope)
      {
        slots.remove(&reference);

        if slots.is_empty() {
          self.by_reference.remove(&scope);
        }
      }

      self.sources[slot] = None;

      self.views[slot] = self.views[Self::MISSING_SLOT as usize].clone();
      self.states[slot] = RenderTextureState::Missing;
      self.blue_free[slot] = false;
      self.references[slot].clear();
      self.generations[slot] = self.generations[slot].wrapping_add(1);
      self.free.push(slot as u32);
      self.is_dirty = true;
    }
  }

  /// Levels the engine drops off a texture's top at `texture_lod` (`get_texture_load_lod`): none to one, one to three,
  /// two at four; a texture its `reduce_lod_texture_list` names a step sooner, none at nought (the address space
  /// allowing).
  pub(crate) fn get_dropped_levels(lod: u32, is_reduced: bool) -> u32 {
    match (is_reduced, lod) {
      (true, 0) | (false, 0..=1) => 0,
      (true, 1..=2) | (false, 2..=3) => 1,
      _ => 2,
    }
  }
}
