use std::ops::Range;

use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphColorAttachment, GraphDepthAttachment, StorageArray, UniformBinding,
};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::particle_batch::ParticleBatch;
use crate::pass::particle_parameters::ParticleParameters;
use crate::pass::particle_pass::ParticlePass;
use crate::pass::particle_vertex::ParticleVertex;
use crate::pass::view_binding::ViewBinding;

/// Quads the vertex buffer holds at first; it doubles past them.
const INITIAL_QUADS: u64 = 4096;

/// A level's particles as one view draws them: the quads of the effects in its camera, far to near, filled by the
/// scene's `LevelParticles::fill` each frame, the batches drawing their colour and the runs drawing their distortion.
pub struct ParticlesView {
  pub vertices: Vec<ParticleVertex>,
  pub batches: Vec<ParticleBatch>,
  /// The distorting effects' quads, far to near.
  pub distortion_runs: Vec<Range<u32>>,
  /// How many effects it draws this frame.
  pub drawn: u32,
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

  /// Writes the quads filled this frame, growing the buffer past them.
  pub fn upload(&mut self, (device, queue): (&wgpu::Device, &wgpu::Queue)) {
    let size: u64 = (self.vertices.len() * size_of::<ParticleVertex>()) as u64;

    if size > self.vertex_buffer.size() {
      self.vertex_buffer = Self::create_storage(device, size.next_power_of_two());
    }

    if !self.vertices.is_empty() {
      queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
    }
  }

  /// Draws the quads filled this frame over the scene, then the distorting ones into the distortion target.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    pass: &'a ParticlePass,
    (targets, lighting, surfaces): (ViewTargetHandles, UniformBinding<LightingUniform>, &'a wgpu::Buffer),
    (view, texture_group): (&'a ViewBinding, &'a wgpu::BindGroup),
  ) {
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
        .record(move |context| pass.record_colour(context, (view, &parameters, texture_group), &self.batches));
    }

    if self.is_distorting() {
      graph
        .add_raster_pass("particle distortion")
        .parameters(&parameters)
        .color(GraphColorAttachment::new(targets.distortion, wgpu::LoadOp::Load))
        .depth(GraphDepthAttachment::new_read_only(targets.depth))
        .record(move |context| {
          pass.record_distortion(context, (view, &parameters, texture_group), &self.distortion_runs)
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
