use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use xrf_material::XraySurfaceDescriptor;
use xrf_visual::SectorSurface;

use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::static_scene_writer::StaticSceneWriter;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_surface_builder::StaticSurfaceBuilder;
use crate::scene::static_scene::static_surface_key::StaticSurfaceKey;
use crate::scene::static_scene::static_surface_pass::StaticSurfacePass;
use crate::scene::static_scene::static_surface_use::StaticSurfaceUse;
use crate::scene::texture::texture_cache::TextureCache;

/// The surfaces every item of a static scene shares: a row each in one buffer, asked for once by key and kept, and the
/// texture and environment slots they sample.
pub struct StaticSurfaces {
  pub buffer: GrowableBuffer,
  /// Every texture slot the surfaces sample, in the order they were first asked for.
  pub texture_slots: BTreeSet<u32>,
  /// Every environment slot its environment-mapped surfaces sample.
  pub environment_slots: BTreeSet<u32>,
  /// Each surface's row and class, once asked for; `None` for one no static draw draws.
  rows: HashMap<StaticSurfaceKey, Option<StaticSurfaceUse>>,
  /// Each impostor surface's row, by its shader table entry.
  impostor_rows: HashMap<u16, u32>,
  /// Rows made, the buffer's and those a writer holds still.
  count: u32,
}

impl StaticSurfaces {
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let mut surfaces: Self = Self {
      buffer: GrowableBuffer::new(device, "static surfaces", wgpu::BufferUsages::STORAGE),
      texture_slots: BTreeSet::new(),
      environment_slots: BTreeSet::new(),
      rows: HashMap::new(),
      impostor_rows: HashMap::new(),
      count: 1,
    };
    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

    // Row zero is never named, so a slot left at zero still reads a valid row.
    surfaces.buffer.append(
      device,
      queue,
      &mut encoder,
      bytemuck::bytes_of(&StaticSurface::default()),
    );
    queue.submit([encoder.finish()]);

    surfaces
  }

  /// Appends the surfaces a writer asked for first, which every item shares from then on.
  pub fn put(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    writer: &StaticSceneWriter,
  ) {
    self
      .buffer
      .append(device, queue, encoder, bytemuck::cast_slice(&writer.surfaces));
  }

  /// A surface's row, its class and the surface passes it takes part in, made into `writer` the first time it is
  /// asked for; none for a surface no static draw draws.
  pub fn resolve(
    &mut self,
    writer: &mut StaticSceneWriter,
    (key, surface, descriptor): (StaticSurfaceKey, &SectorSurface, Option<&XraySurfaceDescriptor>),
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) -> Option<StaticSurfaceUse> {
    if let Some(known) = self.rows.get(&key) {
      return *known;
    }

    let is_model: bool = matches!(key, StaticSurfaceKey::Model { .. });
    let built = StaticSurfaceBuilder::build(surface, descriptor, textures, source);
    let resolved: Option<StaticSurfaceUse> = built.map(|(mut row, class)| {
      if is_model {
        row.flags |= StaticSurface::IS_MODEL;
      }

      let passes: u32 = StaticSurfacePass::mask_of(row.flags);

      StaticSurfaceUse {
        surface: self.push(writer, row),
        class,
        passes,
      }
    });

    self.rows.insert(key, resolved);

    resolved
  }

  /// An impostor surface's row, made into `writer` the first time its shader table entry is asked for.
  pub fn resolve_impostor(
    &mut self,
    writer: &mut StaticSceneWriter,
    surface: &SectorSurface,
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) -> u32 {
    if let Some(row) = self.impostor_rows.get(&surface.shader_id) {
      return *row;
    }

    let row: u32 = self.push(writer, StaticSurfaceBuilder::build_impostor(surface, textures, source));

    self.impostor_rows.insert(surface.shader_id, row);

    row
  }

  /// Adds a row to `writer`, noting the slots it samples; answers its index.
  fn push(&mut self, writer: &mut StaticSceneWriter, surface: StaticSurface) -> u32 {
    self.note_slots(&surface);
    writer.surfaces.push(surface);
    self.count += 1;

    self.count - 1
  }

  /// Notes the slots a surface samples: its textures, and apart from them its environment cube's.
  fn note_slots(&mut self, surface: &StaticSurface) {
    for (index, slot) in surface.textures.iter().enumerate() {
      if index == StaticSurface::ENVIRONMENT {
        if *slot != 0 {
          self.environment_slots.insert(*slot);
        }
      } else if *slot != TextureCache::MISSING_SLOT {
        self.texture_slots.insert(*slot);
      }
    }

    // A terrain's details, bumps and mask are sampled as surely as its base.
    self.texture_slots.extend(
      surface
        .terrain
        .iter_slots()
        .filter(|slot| *slot != TextureCache::MISSING_SLOT),
    );

    if surface.flags & StaticSurface::IS_DISTORTING != 0 {
      self.texture_slots.insert(surface.distortion);
    }
  }
}
