use std::borrow::Borrow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use xrf_error::XrfResult;

use crate::contract::render_texture_group::RenderTextureGroup;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_texture_state::RenderTextureState;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
use crate::host::render_asset_source::{RenderAssetSource, get_texture_name};
use crate::scene::texture::decoded_texture::DecodedTexture;
use crate::scene::texture::texture_role::TextureRole;
use crate::thread::loader_receiver::LoaderReceiver;
use crate::thread::render_workers::RenderWorkers;

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

  /// Loads again every texture of these file names (lowercase, without folder or extension), whatever its scope, and a
  /// terrain's heights where its bump is among them, their sources first forgetting where they resolved: their files
  /// changed. Each keeps what it shows until its new levels land.
  pub fn reload(&mut self, names: &[String]) {
    let mut forgotten: Vec<Arc<dyn RenderAssetSource>> = Vec::new();

    for slot in 1..self.views.len() {
      let (Some(_), Some(source)) = (&self.keys[slot], self.sources[slot].clone()) else {
        continue;
      };

      if self.states[slot] == RenderTextureState::Deferred || !self.is_named(slot, names) {
        continue;
      }

      if !forgotten.iter().any(|known| Arc::ptr_eq(known, &source)) {
        source.forget_textures(names);
        forgotten.push(source);
      }

      self.generations[slot] = self.generations[slot].wrapping_add(1);
      self.load(slot as u32);
    }
  }

  /// Whether a slot reads a file of one of these names: its own, or its bump where it is a terrain's heights.
  fn is_named(&self, slot: usize, names: &[String]) -> bool {
    let reference: &str = &self.references[slot];
    let own: String = get_texture_name(reference);
    let bump: Option<String> = (self.roles[slot] == TextureRole::TerrainHeight).then(|| {
      get_texture_name(&format!(
        "{}{}",
        reference.strip_suffix(TextureRole::HEIGHT_SUFFIX).unwrap_or(reference),
        TextureRole::BUMP_SUFFIX
      ))
    });

    names.iter().any(|name| *name == own || bump.as_ref() == Some(name))
  }

  /// Loads a slot again on a loader thread where the stamp of its file no longer matches what it was read at.
  fn recheck(&self, slot: u32) {
    let index: usize = slot as usize;
    let Some(source) = self.sources[index].clone() else {
      return;
    };
    let (reference, recorded) = (self.references[index].clone(), self.stamps[index]);

    if recorded.is_none() || self.states[index] == RenderTextureState::Deferred {
      return;
    }

    let (sender, generation, role, lod) = (
      self.sender.clone(),
      self.generations[index],
      self.roles[index],
      self.lod,
    );

    self.workers.spawn(move || {
      if source.get_texture_stamp(&reference) != recorded {
        let _ = sender.send(Self::read_slot(&source, (slot, generation), (&reference, role), lod));
      }
    });
  }

  /// Reads and decodes a slot's texture on a loader thread, its levels cut as the engine cuts them at `texture_lod`.
  fn load(&self, slot: u32) {
    let index: usize = slot as usize;
    let Some(source) = self.sources[index].clone() else {
      return;
    };
    let (sender, generation, reference, role, lod) = (
      self.sender.clone(),
      self.generations[index],
      self.references[index].clone(),
      self.roles[index],
      self.lod,
    );

    self.workers.spawn(move || {
      let _ = sender.send(Self::read_slot(&source, (slot, generation), (&reference, role), lod));
    });
  }

  /// What reading a slot's texture comes to, with the stamp of the file it was read from.
  fn read_slot(
    source: &Arc<dyn RenderAssetSource>,
    (slot, generation): (u32, u32),
    (reference, role): (&str, TextureRole),
    lod: u32,
  ) -> TextureLoad {
    // Taken before the read, so a write landing during it is read again later rather than missed.
    let stamp: Option<u64> = source.get_texture_stamp(reference);
    // At full quality the listed textures load whole too, so the list is never read.
    let dropped: u32 = Self::get_dropped_levels(lod, lod > 0 && source.is_texture_reduced(reference));
    let read = |reference: &str| -> XrfResult<Option<DecodedTexture>> {
      source
        .read_texture(reference)?
        .map(|bytes| DecodedTexture::from_dds(&bytes).map(|texture| texture.without_levels(dropped)))
        .transpose()
    };
    let load: Result<Option<DecodedTexture>, String> = match role {
      // A detail's height where the installation brings one, else integrated from its bump.
      TextureRole::TerrainHeight => read(reference)
        .and_then(|own| match own {
          Some(own) => Ok(Some(own)),
          None => {
            let bump: String = format!(
              "{}{}",
              reference.strip_suffix(TextureRole::HEIGHT_SUFFIX).unwrap_or(reference),
              TextureRole::BUMP_SUFFIX
            );

            source
              .read_texture(&bump)?
              .map(|bytes| DecodedTexture::from_bump_heights(&bytes, dropped))
              .transpose()
          }
        })
        .map_err(|error| error.to_string()),
      _ => read(reference).map_err(|error| error.to_string()),
    };

    if let Err(error) = &load {
      log::warn!("Texture '{reference}' cannot be drawn: {error}");
    }

    (slot, generation, load, stamp)
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
      self.references[slot].clear();
      self.generations[slot] = self.generations[slot].wrapping_add(1);
      self.free.push(slot as u32);
      self.is_dirty = true;
    }
  }

  /// Takes what the loaders finished and uploads it, then rebinds the array if a slot changed: up to `UPLOAD_BYTES` a
  /// frame while a level opens (`is_opening`), and as `scheduler` grants them, in a round of their own, once every
  /// level stands.
  pub fn update(
    &mut self,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    scheduler: &mut PeriodicScheduler,
    is_opening: bool,
  ) {
    while let Ok((slot, generation, load, stamp)) = self.receiver.try_recv() {
      if self.generations.get(slot as usize) != Some(&generation) {
        continue;
      }

      self.stamps[slot as usize] = stamp;

      match load {
        Ok(Some(texture)) => self.uploads.push_back((slot, generation, texture)),
        Ok(None) => self.stand_in(slot, RenderTextureState::Missing),
        Err(reason) => self.stand_in(slot, RenderTextureState::Failed { reason }),
      }
    }

    // Those whose slot was asked for again since are dropped.
    self
      .uploads
      .retain(|(slot, generation, _)| self.generations[*slot as usize] == *generation);

    if is_opening {
      let mut spent: u64 = 0;

      // The first upload of a frame always goes, so one texture larger than the budget is not held forever.
      while spent < UPLOAD_BYTES
        && let Some((slot, _, texture)) = self.uploads.pop_front()
      {
        spent += texture.get_size();
        self.upload((device, queue), slot, &texture);
      }

      self.waited = 0;
    } else if !self.uploads.is_empty() {
      let count: usize = self.uploads.len().min(UPLOADS_OFFERED);
      let tickets: Vec<PeriodicTicket> = self
        .uploads
        .iter()
        .take(count)
        .map(|(_, _, texture)| {
          scheduler.offer(PeriodicJob::new_waiting(
            &UPLOAD_PASSES,
            UPLOAD_SEED,
            texture.get_size() as f32,
            (self.waited, UPLOAD_WAIT_MOST),
          ))
        })
        .collect();

      scheduler.grant();

      let offered: Vec<(u32, u32, DecodedTexture)> = self.uploads.drain(..count).collect();
      let mut kept: Vec<(u32, u32, DecodedTexture)> = Vec::new();

      for ((slot, generation, texture), ticket) in offered.into_iter().zip(tickets) {
        if scheduler.is_granted(ticket) {
          self.upload((device, queue), slot, &texture);
        } else {
          kept.push((slot, generation, texture));
        }
      }

      self.waited = if kept.is_empty() { 0 } else { self.waited + 1 };

      for item in kept.into_iter().rev() {
        self.uploads.push_front(item);
      }
    }

    if self.is_dirty {
      self.is_dirty = false;
      self.bind_group = Self::create_bind_group(device, &self.layout, &self.sampler, &self.views);
    }
  }

  /// Uploads a loaded texture into its slot.
  fn upload(&mut self, (device, queue): (&wgpu::Device, &wgpu::Queue), slot: u32, texture: &DecodedTexture) {
    // One view the device refuses would make the whole bindless group invalid, and every surface with it.
    let limits: wgpu::Limits = device.limits();

    if !texture.is_within(&limits) {
      let reason: String = format!(
        "{} by {} texels, past the {} a side the device holds",
        texture.width, texture.height, limits.max_texture_dimension_2d
      );

      self.stand_in(slot, RenderTextureState::Failed { reason });

      return;
    }

    // The bindless array holds two dimensional textures alone: a cube or a volume named as a level texture draws its
    // first face.
    // todo: refuse a cube or a volume named as a level texture as failed, rather than drawing its first face.
    self.views[slot as usize] = texture.upload_flat(device, queue, "level texture");
    self.states[slot as usize] = RenderTextureState::Loaded {
      width: texture.width,
      height: texture.height,
      levels: texture.levels.len() as u32,
      layout: texture.layout.clone(),
      is_expanded: texture.is_expanded,
    };
    self.is_dirty = true;
  }

  /// Stands something in for a texture that cannot be had: a checker for a base, so the gap is seen, else neutral.
  fn stand_in(&mut self, slot: u32, state: RenderTextureState) {
    if self.roles[slot as usize] == TextureRole::Base {
      self.views[slot as usize] = self.checker.clone();
      self.is_dirty = true;
    }

    self.states[slot as usize] = state;
  }

  /// Magenta and near black squares, as the WebGPU viewers stand in for a texture they cannot have.
  fn create_checker(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    use wgpu::util::DeviceExt;

    let texels: Vec<u8> = (0..CHECKER_SIZE * CHECKER_SIZE)
      .flat_map(|index| {
        let (x, y): (u32, u32) = (
          index % CHECKER_SIZE / CHECKER_SQUARE,
          index / CHECKER_SIZE / CHECKER_SQUARE,
        );

        if (x + y) % 2 == 0 {
          [255, 0, 255, 255]
        } else {
          [16, 16, 16, 255]
        }
      })
      .collect();

    device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("checker"),
          size: wgpu::Extent3d {
            width: CHECKER_SIZE,
            height: CHECKER_SIZE,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: wgpu::TextureFormat::Rgba8Unorm,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        &texels,
      )
      .create_view(&Default::default())
  }

  fn create_solid(device: &wgpu::Device, queue: &wgpu::Queue, color: [u8; 4]) -> wgpu::TextureView {
    use wgpu::util::DeviceExt;

    device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("solid"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: wgpu::TextureFormat::Rgba8Unorm,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        &color,
      )
      .create_view(&Default::default())
  }

  /// Samples the textures with an anisotropy clamp, one for trilinear, made again where it changed.
  pub fn set_anisotropy(&mut self, device: &wgpu::Device, anisotropy: u16) {
    if anisotropy == self.anisotropy {
      return;
    }

    self.anisotropy = anisotropy;
    self.sampler = Self::create_sampler(device, anisotropy);
    self.bind_group = Self::create_bind_group(device, &self.layout, &self.sampler, &self.views);
  }

  /// Trilinear, anisotropic up to `anisotropy`, repeating, as the engine samples a level's surfaces.
  fn create_sampler(device: &wgpu::Device, anisotropy: u16) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("textures"),
      address_mode_u: wgpu::AddressMode::Repeat,
      address_mode_v: wgpu::AddressMode::Repeat,
      address_mode_w: wgpu::AddressMode::Repeat,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::MipmapFilterMode::Linear,
      anisotropy_clamp: anisotropy.max(1),
      ..Default::default()
    })
  }

  /// Binds every slot, the ones never filled to the missing texture, since an array is bound whole.
  fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    views: &[wgpu::TextureView],
  ) -> wgpu::BindGroup {
    // Only the slots asked for: the array is partially bound, and every view bound is tracked by each pass binding it.
    let bound: Vec<&wgpu::TextureView> = views.iter().collect();

    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("textures"),
      layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureViewArray(&bound),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(sampler),
        },
      ],
    })
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
