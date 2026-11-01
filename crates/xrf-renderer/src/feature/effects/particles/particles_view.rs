use std::ops::Range;

use xrf_particles::ParticleEffectInstance;
use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphDepthAttachment, StorageArray, UniformBinding};

use crate::feature::effects::particles::particle_batch::ParticleBatch;
use crate::feature::effects::particles::particle_parameters::ParticleParameters;
use crate::feature::effects::particles::particle_pass::ParticlePass;
use crate::feature::view::view_feature::ViewFeature;
use crate::frame::frame_encoder::FrameEncoder;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;
use crate::scene::level::particles::particle_blend::ParticleBlend;
use crate::scene::level::particles::particle_sprite::ParticleSprite;
use crate::scene::level::particles::particle_surface::ParticleSurface;
use crate::scene::level::particles::particle_vertex::ParticleVertex;
use crate::scene::level::view::record::view_record::ViewRecord;

/// Quads the vertex buffer holds at first; it doubles past them.
const INITIAL_QUADS: u64 = 4096;

/// A level's particles as one view draws them: the quads of the effects in its camera, far to near, filled by the
/// scene's `LevelParticles::fill` each frame, the batches drawing their colour and the runs drawing their distortion.
pub struct ParticlesView {
  vertices: Vec<ParticleVertex>,
  /// The blending effects' quads, far to near, a batch each run of one equation.
  batches: Vec<ParticleBatch>,
  /// The distorting effects' quads, far to near.
  distortion_runs: Vec<Range<u32>>,
  /// How many effects it draws this frame.
  drawn: u32,
  vertex_buffer: wgpu::Buffer,
}

impl ParticlesView {
  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      vertices: Vec::new(),
      batches: Vec::new(),
      distortion_runs: Vec::new(),
      drawn: 0,
      vertex_buffer: Self::create_storage(
        device,
        INITIAL_QUADS * u64::from(ParticleVertex::CORNERS) * size_of::<ParticleVertex>() as u64,
      ),
    }
  }

  /// Forgets last frame's quads.
  pub fn clear(&mut self) {
    self.vertices.clear();
    self.batches.clear();
    self.distortion_runs.clear();
    self.drawn = 0;
  }

  /// Adds an effect's quads after those added before, as `sprite` faces them with its `surface`: into its equation's
  /// batch where it blends, into a distortion run where it distorts, as the surface draws (`drawing`).
  pub fn push_effect(
    &mut self,
    sprite: &ParticleSprite,
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

    if is_distorting {
      match self.distortion_runs.last_mut() {
        Some(last) if last.end == first => last.end = end,
        _ => self.distortion_runs.push(first..end),
      }
    }

    self.drawn += 1;
  }

  /// How many effects it draws this frame.
  pub fn get_drawn(&self) -> u32 {
    self.drawn
  }

  /// Writes the quads filled this frame, growing the buffer past them.
  pub fn upload(&mut self, device: &wgpu::Device, encoder: &mut FrameEncoder) {
    let size: u64 = (self.vertices.len() * size_of::<ParticleVertex>()) as u64;

    if size > self.vertex_buffer.size() {
      self.vertex_buffer = Self::create_storage(device, size.next_power_of_two());
    }

    if !self.vertices.is_empty() {
      encoder.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
    }
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
    let pass: &'a ParticlePass = &passes.effects.particles;
    let lighting: UniformBinding<LightingUniform> = lit.lighting;
    let parameters: ParticleParameters = ParticleParameters {
      vertices: StorageArray::new(bindings.import_buffer(&mut *graph, "particle vertices", &self.vertex_buffer)),
      surfaces: StorageArray::new(bindings.import_buffer(&mut *graph, "particle surfaces", surfaces)),
      lighting,
      depth_target: targets.depth,
      clamped_sampler: pass.get_clamped_sampler(),
    };

    if !self.batches.is_empty() {
      graph
        .add_raster_pass("particles")
        .parameters(&parameters)
        .color(GraphColorAttachment::new(targets.scene, wgpu::LoadOp::Load))
        .depth(GraphDepthAttachment::new_read_only(targets.depth))
        .record(move |context| pass.record_colour(context, view, &parameters, texture_group, &self.batches));
    }

    if self.is_distorting() {
      graph
        .add_raster_pass("particle distortion")
        .parameters(&parameters)
        .color(GraphColorAttachment::new(targets.distortion, wgpu::LoadOp::Load))
        .depth(GraphDepthAttachment::new_read_only(targets.depth))
        .record(move |context| {
          pass.record_distortion(context, view, &parameters, texture_group, &self.distortion_runs)
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
    self.vertex_buffer.size()
  }

  fn release(&mut self) {
    self.clear();
  }
}
