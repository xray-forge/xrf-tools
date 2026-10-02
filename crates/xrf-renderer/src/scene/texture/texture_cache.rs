use std::collections::{HashMap, VecDeque};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_texture_state::RenderTextureState;
use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::texture::decoded_texture::DecodedTexture;
use crate::scene::texture::texture_role::TextureRole;

/// The slot a surface binds where it names no texture of a kind: mid grey, never sampled by a surface without one.
pub const MISSING_SLOT: u32 = 0;

/// Texels a side of the checker standing in for a base texture that cannot be had, and of each of its squares.
const CHECKER_SIZE: u32 = 16;
const CHECKER_SQUARE: u32 = 2;

/// Bytes of texture uploaded at most each frame, so an open spreads its uploads rather than stalling a frame.
const UPLOAD_BYTES: u64 = 64 * 1024 * 1024;

/// Slots a bindless array holds at most, below what any adapter meeting the baseline offers.
const MAX_SLOTS: u32 = 16_384;

/// Environment cubes the scenes' surfaces may mix toward, the first slot standing for none.
pub const ENVIRONMENT_SLOTS: u32 = 16;

/// What a texture load came to, sent from a loader thread.
type TextureLoad = (u32, Result<Option<DecodedTexture>, String>);

/// Every texture the renderer's scenes sample, once each by reference, in one bindless array: a material names its
/// texture by slot, so draws are never split by texture.
pub struct TextureCache {
  layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
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
  by_reference: HashMap<String, u32>,
  sender: Sender<TextureLoad>,
  receiver: Receiver<TextureLoad>,
  /// Loaded, waiting for a frame's upload budget.
  uploads: VecDeque<(u32, DecodedTexture)>,
  capacity: u32,
  is_dirty: bool,
  /// The cubes surfaces named as their environment, each slot past the first in turn; loaded as weather textures are.
  environments: Vec<String>,
}

impl TextureCache {
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    // The environment cubes a composited draw binds beside the array share the stage's budget.
    let capacity: u32 = device
      .limits()
      .max_binding_array_elements_per_shader_stage
      .saturating_sub(ENVIRONMENT_SLOTS)
      .clamp(1, MAX_SLOTS);
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("textures"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: NonZeroU32::new(capacity),
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });
    // Trilinear with eight times anisotropy, repeating, as the engine samples a level's surfaces.
    let sampler: wgpu::Sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("textures"),
      address_mode_u: wgpu::AddressMode::Repeat,
      address_mode_v: wgpu::AddressMode::Repeat,
      address_mode_w: wgpu::AddressMode::Repeat,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::MipmapFilterMode::Linear,
      anisotropy_clamp: 8,
      ..Default::default()
    });
    let missing: wgpu::TextureView = Self::create_solid(device, queue, [128, 128, 128, 255]);
    let neutral: Vec<wgpu::TextureView> = TextureRole::ALL
      .into_iter()
      .map(|role| Self::create_solid(device, queue, role.get_neutral()))
      .collect();
    let checker: wgpu::TextureView = Self::create_checker(device, queue);
    let (sender, receiver) = channel();
    let views: Vec<wgpu::TextureView> = vec![missing];
    let bind_group: wgpu::BindGroup = Self::create_bind_group(device, &layout, &sampler, &views);

    Self {
      layout,
      sampler,
      bind_group,
      views,
      states: vec![RenderTextureState::Missing],
      references: vec![String::new()],
      roles: vec![TextureRole::Detail],
      neutral,
      checker,
      by_reference: HashMap::new(),
      sender,
      receiver,
      uploads: VecDeque::new(),
      capacity,
      is_dirty: false,
      environments: Vec::new(),
    }
  }

  pub fn get_layout(&self) -> &wgpu::BindGroupLayout {
    &self.layout
  }

  pub fn get_bind_group(&self) -> &wgpu::BindGroup {
    &self.bind_group
  }

  /// The slot a texture reference samples from, loading it on a loader thread the first time it is asked for; until
  /// it loads, its role's neutral stands in.
  pub fn request(&mut self, reference: &str, role: TextureRole, source: &Arc<dyn RenderAssetSource>) -> u32 {
    if reference.is_empty() {
      return MISSING_SLOT;
    }

    if let Some(&slot) = self.by_reference.get(reference) {
      return slot;
    }

    let slot: u32 = self.views.len() as u32;

    if slot >= self.capacity {
      // todo: Grow the bindless array, or evict, once a scene names more textures than the adapter binds at once.
      log::warn!(
        "Texture '{reference}' exceeds the {} the renderer binds at once",
        self.capacity
      );

      return MISSING_SLOT;
    }

    self.views.push(self.neutral[role as usize].clone());
    self.states.push(RenderTextureState::Loading);
    self.references.push(reference.to_string());
    self.roles.push(role);
    self.is_dirty = true;
    self.by_reference.insert(reference.to_string(), slot);

    let (sender, source, reference) = (self.sender.clone(), Arc::clone(source), reference.to_string());

    rayon::spawn(move || {
      let load: Result<Option<DecodedTexture>, String> = source
        .read_texture(&reference)
        .and_then(|bytes| bytes.map(|bytes| DecodedTexture::from_dds(&bytes)).transpose())
        .map_err(|error| error.to_string());

      if let Err(error) = &load {
        log::warn!("Texture '{reference}' cannot be drawn: {error}");
      }

      let _ = sender.send((slot, load));
    });

    slot
  }

  /// The environment slot a cube reference is sampled from, `0` for none or for one past the slots there are.
  pub fn request_environment(&mut self, reference: &str) -> u32 {
    if reference.is_empty() {
      return 0;
    }

    if let Some(index) = self.environments.iter().position(|it| it == reference) {
      return index as u32 + 1;
    }

    if self.environments.len() as u32 + 1 >= ENVIRONMENT_SLOTS {
      log::warn!("Environment '{reference}' exceeds the {ENVIRONMENT_SLOTS} the renderer binds at once");

      return 0;
    }

    self.environments.push(reference.to_owned());

    self.environments.len() as u32
  }

  /// The cubes asked for as environments, by slot from the second.
  pub fn list_environments(&self) -> &[String] {
    &self.environments
  }

  /// Of some slots, how many are uploaded or given up on.
  pub fn count_settled<'a>(&self, slots: impl IntoIterator<Item = &'a u32>) -> u32 {
    slots
      .into_iter()
      .filter(|slot| !matches!(self.states.get(**slot as usize), Some(RenderTextureState::Loading)))
      .count() as u32
  }

  /// What became of some slots' textures.
  pub fn describe<'a>(&self, slots: impl IntoIterator<Item = &'a u32>) -> Vec<RenderTextureReport> {
    slots
      .into_iter()
      .filter_map(|slot| {
        Some(RenderTextureReport {
          reference: self.references.get(*slot as usize)?.clone(),
          state: self.states.get(*slot as usize)?.clone(),
        })
      })
      .collect()
  }

  /// Takes what the loaders finished and uploads it, within a frame's budget, then rebinds the array if a slot changed.
  pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    while let Ok((slot, load)) = self.receiver.try_recv() {
      match load {
        Ok(Some(texture)) => self.uploads.push_back((slot, texture)),
        Ok(None) => self.stand_in(slot, RenderTextureState::Missing),
        Err(reason) => self.stand_in(slot, RenderTextureState::Failed { reason }),
      }
    }

    let mut spent: u64 = 0;

    // The first upload of a frame always goes, so one texture larger than the budget is not held forever.
    while spent < UPLOAD_BYTES
      && let Some((slot, texture)) = self.uploads.pop_front()
    {
      spent += texture.get_size();
      self.views[slot as usize] = Self::upload(device, queue, &texture);
      self.states[slot as usize] = RenderTextureState::Loaded {
        width: texture.width,
        height: texture.height,
        levels: texture.levels.len() as u32,
        layout: texture.layout.clone(),
        is_expanded: texture.is_expanded,
      };
      self.is_dirty = true;
    }

    if self.is_dirty {
      self.is_dirty = false;
      self.bind_group = Self::create_bind_group(device, &self.layout, &self.sampler, &self.views);
    }
  }

  /// Stands something in for a texture that cannot be had: a checker for a base, so the gap is seen, else neutral.
  fn stand_in(&mut self, slot: u32, state: RenderTextureState) {
    if self.roles[slot as usize] == TextureRole::Base {
      self.views[slot as usize] = self.checker.clone();
      self.is_dirty = true;
    }

    self.states[slot as usize] = state;
  }

  fn upload(device: &wgpu::Device, queue: &wgpu::Queue, texture: &DecodedTexture) -> wgpu::TextureView {
    let size: wgpu::Extent3d = wgpu::Extent3d {
      width: texture.width,
      height: texture.height,
      depth_or_array_layers: 1,
    };
    let gpu: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: None,
      size,
      mip_level_count: texture.levels.len() as u32,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: texture.format,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });
    let (block_width, block_height) = texture.format.block_dimensions();
    let block_bytes: u32 = texture.format.block_copy_size(None).unwrap_or(4);

    for (level, bytes) in texture.levels.iter().enumerate() {
      let physical: wgpu::Extent3d = size
        .mip_level_size(level as u32, wgpu::TextureDimension::D2)
        .physical_size(texture.format);

      queue.write_texture(
        wgpu::TexelCopyTextureInfo {
          texture: &gpu,
          mip_level: level as u32,
          origin: wgpu::Origin3d::ZERO,
          aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
          offset: 0,
          bytes_per_row: Some(physical.width / block_width * block_bytes),
          rows_per_image: Some(physical.height / block_height),
        },
        physical,
      );
    }

    gpu.create_view(&Default::default())
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
}
