use std::ops::Range;

use glam::{Mat4, Vec3};
use xrf_particles::ParticleEffectInstance;
use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphDepthAttachment, StorageArray, UniformBinding};

use crate::feature::effects::over_scene::add_over_scene;
use crate::feature::effects::particles::particle_batch::ParticleBatch;
use crate::feature::effects::particles::particle_parameters::ParticleParameters;
use crate::feature::effects::particles::particle_pass::ParticlePass;
use crate::feature::view::view_feature::ViewFeature;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::targets::screen_tiles::ScreenTiles;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;
use crate::scene::level::particles::particle_blend::ParticleBlend;
use crate::scene::level::particles::particle_sprite::ParticleSprite;
use crate::scene::level::particles::particle_surface::ParticleSurface;
use crate::scene::level::particles::particle_vertex::ParticleVertex;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::texture::texture_cache::TextureCache;

/// Quads the vertex buffer holds at first; it doubles past them.
const INITIAL_QUADS: u64 = 4096;

/// A level's particles as one view draws them: the quads of the effects in its camera, far to near, filled by the
/// scene's `LevelParticles::fill` each frame, the batches drawing their colour and the runs drawing their distortion
/// where it reaches the screen.
#[derive(Default)]
pub struct ParticlesView {
  vertices: Vec<ParticleVertex>,
  /// The blending effects' quads, far to near, a batch each run of one equation.
  batches: Vec<ParticleBatch>,
  /// The quads of the distorting effects with one on the screen, far to near.
  distortion_runs: Vec<Range<u32>>,
  /// The tiles of the frame those quads touch.
  distortion_tiles: ScreenTiles,
  /// The texture slots those effects draw into the distortion target, once each.
  distortion_slots: Vec<u32>,
  /// How many effects it draws this frame.
  drawn: u32,
  /// The quads drawn, made with the first quads to draw; none until then, or once let go.
  vertex_buffer: Option<wgpu::Buffer>,
}

impl ParticlesView {
  /// Bytes one quad takes in the vertex buffer.
  const QUAD_BYTES: u64 = ParticleVertex::CORNERS as u64 * size_of::<ParticleVertex>() as u64;

  /// Forgets last frame's quads.
  pub fn clear(&mut self) {
    self.vertices.clear();
    self.batches.clear();
    self.distortion_runs.clear();
    self.distortion_tiles = ScreenTiles::default();
    self.distortion_slots.clear();
    self.drawn = 0;
  }

  /// Adds an effect's quads after those added before, as `sprite` faces them with its `surface`: into its equation's
  /// batch where it blends, into a distortion run where it distorts, as the surface draws (`drawing`), and one of its
  /// quads reaches the screen of `view_projection`.
  pub fn push_effect(
    &mut self,
    sprite: &ParticleSprite,
    view_projection: Mat4,
    effect: &ParticleEffectInstance,
    surface: u32,
    drawing: &ParticleSurface,
  ) {
    let (blend, is_distorting): (Option<ParticleBlend>, bool) = (drawing.blend, drawing.is_distorting);
    let first: u32 = self.vertices.len() as u32 / ParticleVertex::CORNERS;

    sprite.push(effect, surface, &mut self.vertices);

    let end: u32 = self.vertices.len() as u32 / ParticleVertex::CORNERS;

    if let Some(blend) = blend {
      match self.batches.last_mut() {
        Some(last) if last.blend == blend && last.first + last.count == first => last.count = end - last.first,
        _ => self.batches.push(ParticleBatch {
          blend,
          first,
          count: end - first,
        }),
      }
    }

    if is_distorting && self.cover_distortion(view_projection, first..end) {
      match self.distortion_runs.last_mut() {
        Some(last) if last.end == first => last.end = end,
        _ => self.distortion_runs.push(first..end),
      }

      if !self.distortion_slots.contains(&drawing.record.distortion) {
        self.distortion_slots.push(drawing.record.distortion);
      }
    }

    self.drawn += 1;
  }

  /// Marks the tiles `quads` touch on the screen of `view_projection`; whether one does.
  fn cover_distortion(&mut self, view_projection: Mat4, quads: Range<u32>) -> bool {
    let corners: usize = ParticleVertex::CORNERS as usize;
    let mut tiles: ScreenTiles = ScreenTiles::default();

    for quad in self.vertices[quads.start as usize * corners..quads.end as usize * corners].chunks_exact(corners) {
      let points: [Vec3; 4] = std::array::from_fn(|corner| quad[corner].position);

      tiles.cover_hull(view_projection, &points);
    }

    self.distortion_tiles.cover_tiles(&tiles);

    !tiles.is_empty()
  }

  /// How many effects it draws this frame.
  pub fn get_drawn(&self) -> u32 {
    self.drawn
  }

  /// Writes the quads filled this frame, growing the buffer past them up to the most one binding may hold; the nearest
  /// quads past that are not drawn.
  pub fn upload(&mut self, device: &wgpu::Device, encoder: &mut FrameEncoder) {
    let most_quads: u64 = device.limits().max_storage_buffer_binding_size / Self::QUAD_BYTES;

    self.keep_quads(u32::try_from(most_quads).unwrap_or(u32::MAX));

    if self.vertices.is_empty() {
      return;
    }

    let size: u64 = (self.vertices.len() * size_of::<ParticleVertex>()) as u64;

    if self.vertex_buffer.as_ref().is_none_or(|buffer| size > buffer.size()) {
      let capacity: u64 = size
        .next_power_of_two()
        .max(INITIAL_QUADS * Self::QUAD_BYTES)
        .min(most_quads * Self::QUAD_BYTES);

      self.vertex_buffer = Some(Self::create_storage(device, capacity));
    }

    if let Some(buffer) = &self.vertex_buffer {
      encoder.write_buffer(buffer, 0, bytemuck::cast_slice(&self.vertices));
    }
  }

  /// Keeps the first `most` quads, far to near, cutting the batches and the distortion runs at them.
  fn keep_quads(&mut self, most: u32) {
    if self.vertices.len() <= most as usize * ParticleVertex::CORNERS as usize {
      return;
    }

    self.vertices.truncate(most as usize * ParticleVertex::CORNERS as usize);
    self.batches.retain_mut(|batch| {
      batch.count = batch.count.min(most.saturating_sub(batch.first));

      batch.count > 0
    });
    self.distortion_runs.retain_mut(|run| {
      run.end = run.end.min(most);

      run.start < run.end
    });
  }

  /// Draws the quads filled this frame over the scene, then the distorting ones into the distortion target, each
  /// surface's record read from `surfaces`.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes,
      view,
      textures: texture_group,
      handles: targets,
      lit,
      ..
    }: ViewRecord<'a>,
    surfaces: &'a wgpu::Buffer,
  ) {
    let Some(vertex_buffer) = &self.vertex_buffer else {
      return;
    };
    let pass: &'a ParticlePass = &passes.effects.particles;
    let lighting: UniformBinding<LightingUniform> = lit.lighting;
    let parameters: ParticleParameters<'_> = ParticleParameters {
      vertices: StorageArray::new(bindings.import_buffer(&mut *graph, "particle vertices", vertex_buffer)),
      surfaces: StorageArray::new(bindings.import_buffer(&mut *graph, "particle surfaces", surfaces)),
      lighting,
      depth_target: targets.depth,
      clamped_sampler: pass.get_clamped_sampler(),
    };

    if !self.batches.is_empty() {
      add_over_scene(&mut *graph, targets, "particles")
        .parameters(&parameters)
        .record(move |context| pass.record_colour(context, view, &parameters, texture_group, &self.batches));
    }

    if self.is_distorting() {
      graph
        .add_raster_pass("particle distortion")
        .parameters(&parameters)
        .color(GraphColorAttachment::new(targets.distortion, wgpu::LoadOp::Load))
        .depth(GraphDepthAttachment::new_read_only(targets.depth))
        .record(move |context| {
          pass.record_distortion(context, view, &parameters, texture_group, &self.distortion_runs);
        });
    }
  }

  /// Whether this frame has particles to draw, as `add_passes` would draw them.
  pub fn is_drawing(&self) -> bool {
    !self.batches.is_empty() || self.is_distorting()
  }

  /// Whether this frame's particles draw into the distortion target.
  pub fn is_distorting(&self) -> bool {
    !self.distortion_runs.is_empty()
  }

  /// The tiles of the frame this frame's particles draw into the distortion target in.
  pub fn get_distortion_tiles(&self) -> &ScreenTiles {
    &self.distortion_tiles
  }

  /// Whether this frame's particles may blur the frame: one drawn into the distortion target whose map holds some blue
  /// or is not yet up (`TextureCache::is_blue_free`).
  pub fn is_blurring(&self, textures: &TextureCache) -> bool {
    self.distortion_slots.iter().any(|slot| !textures.is_blue_free(*slot))
  }

  fn create_storage(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("particle vertices"),
      size,
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    })
  }
}

impl ViewFeature for ParticlesView {
  /// Its quads' buffer.
  fn get_memory(&self) -> u64 {
    self.vertex_buffer.as_ref().map_or(0, wgpu::Buffer::size)
  }

  fn release(&mut self) {
    self.clear();
    self.vertex_buffer = None;
  }
}
