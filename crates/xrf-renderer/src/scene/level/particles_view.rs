use std::ops::Range;

use xrf_renderer_core::{FrameGraph, GraphColorAttachment, GraphDepthAttachment};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::pass::particle_batch::ParticleBatch;
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
  /// Bumped whenever the vertex buffer is replaced, which the bind group follows.
  vertices_generation: u64,
  /// The bind group, with the targets' epoch and the vertex and surface buffers' generations it binds.
  group: Option<((u64, u64, u64), wgpu::BindGroup)>,
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
      vertices_generation: 0,
      group: None,
    }
  }

  /// Forgets last frame's quads.
  pub fn clear(&mut self) {
    self.vertices.clear();
    self.batches.clear();
    self.distortion_runs.clear();
    self.drawn = 0;
  }

  /// Writes the quads filled this frame, growing the buffer past them, and binds what the pass draws them with: the
  /// scene's `surfaces` and how many times they were made.
  pub fn upload(
    &mut self,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    pass: &ParticlePass,
    (lighting, surfaces): (&wgpu::Buffer, (&wgpu::Buffer, u64)),
    (targets, targets_epoch): (&ViewTargets, u64),
  ) {
    let size: u64 = (self.vertices.len() * size_of::<ParticleVertex>()) as u64;

    if size > self.vertex_buffer.size() {
      self.vertex_buffer = Self::create_storage(device, size.next_power_of_two());
      self.vertices_generation += 1;
    }

    if !self.vertices.is_empty() {
      queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
    }

    let (surfaces, surfaces_generation) = surfaces;
    let key: (u64, u64, u64) = (targets_epoch, self.vertices_generation, surfaces_generation);

    if self.group.as_ref().is_none_or(|(bound, _)| *bound != key) {
      self.group = Some((
        key,
        pass.create_bind_group(device, &self.vertex_buffer, surfaces, lighting, targets),
      ));
    }
  }

  /// Draws the quads filled this frame over the scene, then the distorting ones into the distortion target.
  pub fn add_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    pass: &'a ParticlePass,
    targets: ViewTargetHandles,
    (view, texture_group): (&'a ViewBinding, &'a wgpu::BindGroup),
  ) {
    let Some((_, group)) = &self.group else {
      return;
    };

    if !self.batches.is_empty() {
      graph
        .add_raster_pass("particles")
        .color(GraphColorAttachment::new(targets.scene, wgpu::LoadOp::Load))
        .depth(GraphDepthAttachment::new_read_only(targets.depth))
        .record(move |context| pass.record_colour(context.get_pass(), (view, group, texture_group), &self.batches));
    }

    if self.is_distorting() {
      graph
        .add_raster_pass("particle distortion")
        .color(GraphColorAttachment::new(targets.distortion, wgpu::LoadOp::Load))
        .depth(GraphDepthAttachment::new_read_only(targets.depth))
        .record(move |context| {
          pass.record_distortion(context.get_pass(), (view, group, texture_group), &self.distortion_runs)
        });
    }
  }

  /// Whether this frame has particles to draw, as `add_passes` would draw them.
  pub fn is_drawing(&self) -> bool {
    self.group.is_some() && (!self.batches.is_empty() || self.is_distorting())
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
