use crate::frame::frame_encoder::FrameEncoder;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_geometry::StaticGeometry;
use crate::scene::static_scene::static_region::StaticRegion;

/// What a frame's culls of a static scene write into: the visible list, its run a draw list, the candidates the early
/// phase sets aside, and the draw arguments, reset from templates each frame; sized by the entries each draw list may
/// need. Every view of the scene culls into the same buffers, in the order the frame declares them.
pub struct StaticCullScratch {
  pub lists: GrowableBuffer,
  /// Clusters the early phase set aside, for the late phase to test again.
  pub candidates: GrowableBuffer,
  pub regions: wgpu::Buffer,
  pub args: wgpu::Buffer,
  /// Each batch's late draw arguments, then the late phase's dispatch and the candidates' count.
  pub late: wgpu::Buffer,
  /// The late phase's dispatch, copied out of `late` so the dispatch reads no buffer it binds.
  pub late_dispatch: wgpu::Buffer,
  /// Every batch's draw arguments as a cull starts them, copied into a view's own before each cull of it in a frame.
  pub args_template: wgpu::Buffer,
  /// The late draw arguments as the main cull starts them.
  late_template: wgpu::Buffer,
  /// Entries of the visible list each draw list may need: its clusters, each counted once per place drawing it.
  capacities: [u32; StaticBatch::LIST_COUNT],
  /// The capacities the regions and the templates were last written for.
  written: Option<[u32; StaticBatch::LIST_COUNT]>,
  /// Entries the visible list and the candidates hold.
  list_capacity: u32,
}

impl StaticCullScratch {
  /// Where the cull's counts follow every draw list's arguments in their buffer.
  pub const STATS_OFFSET: u64 = (StaticBatch::LIST_COUNT * 16) as u64;

  /// Where the impostors' draw arguments follow the cull's counts.
  pub const IMPOSTOR_ARGS_OFFSET: u64 = Self::STATS_OFFSET + 16;

  /// Where the late phase's dispatch follows every draw list's late arguments in theirs.
  pub const LATE_DISPATCH_OFFSET: u64 = (StaticBatch::LIST_COUNT * 16) as u64;

  /// Bytes a cull's draw arguments take.
  pub const ARGS_SIZE: u64 = Self::IMPOSTOR_ARGS_OFFSET + 16;

  pub fn new(device: &wgpu::Device) -> Self {
    let storage: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE;
    let drawn: wgpu::BufferUsages =
      storage | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
    let buffer = |label: &str, size: u64, usage: wgpu::BufferUsages| {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
      })
    };
    let template: wgpu::BufferUsages = wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST;

    Self {
      lists: GrowableBuffer::new(device, "static lists", storage),
      candidates: GrowableBuffer::new(device, "static candidates", storage),
      regions: buffer(
        "static regions",
        (StaticBatch::LIST_COUNT * size_of::<StaticRegion>()) as u64,
        storage | wgpu::BufferUsages::COPY_DST,
      ),
      args: buffer("static draw arguments", Self::ARGS_SIZE, drawn),
      late: buffer("static late draw arguments", Self::LATE_DISPATCH_OFFSET + 16, drawn),
      late_dispatch: buffer(
        "static late dispatch",
        12,
        wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
      ),
      args_template: buffer("static draw arguments template", Self::ARGS_SIZE, template),
      late_template: buffer(
        "static late draw arguments template",
        Self::LATE_DISPATCH_OFFSET + 16,
        template,
      ),
      capacities: [0; StaticBatch::LIST_COUNT],
      written: None,
      list_capacity: 0,
    }
  }

  pub fn add_capacities(&mut self, capacities: &[u32; StaticBatch::LIST_COUNT]) {
    for (total, added) in self.capacities.iter_mut().zip(capacities) {
      *total += added;
    }
  }

  pub fn remove_capacities(&mut self, capacities: &[u32; StaticBatch::LIST_COUNT]) {
    for (total, removed) in self.capacities.iter_mut().zip(capacities) {
      *total -= removed;
    }
  }

  /// Resets each draw list's arguments empty from their templates for a frame's cull to fill; the runs of the visible
  /// list and the templates are written again only when the capacities changed.
  pub fn prepare(&mut self, device: &wgpu::Device, encoder: &mut FrameEncoder) {
    if self.written != Some(self.capacities) {
      self.write(device, encoder);
    }

    self.reset_args(encoder, &self.args);
    encoder.copy_buffer_to_buffer(&self.late_template, 0, &self.late, 0, Self::LATE_DISPATCH_OFFSET + 16);
  }

  /// Resets a cull's draw arguments empty, as every draw list's start a frame.
  pub fn reset_args(&self, encoder: &mut wgpu::CommandEncoder, args: &wgpu::Buffer) {
    encoder.copy_buffer_to_buffer(&self.args_template, 0, args, 0, Self::ARGS_SIZE);
  }

  /// Entries the visible list holds, and the candidates as many.
  pub fn get_list_capacity(&self) -> u32 {
    self.list_capacity
  }

  /// Writes each draw list's run of the visible list and the templates its draw arguments are reset from.
  fn write(&mut self, device: &wgpu::Device, encoder: &mut FrameEncoder) {
    let mut base: u32 = 0;
    let mut regions: Vec<StaticRegion> = Vec::with_capacity(StaticBatch::LIST_COUNT);
    let mut args: Vec<u32> = Vec::with_capacity(StaticBatch::LIST_COUNT * 4 + 8);
    let mut late: Vec<u32> = Vec::with_capacity(StaticBatch::LIST_COUNT * 4 + 4);

    for capacity in self.capacities {
      regions.push(StaticRegion { base, capacity });
      // Every visible cluster is an instance of one cluster's vertices, its entry found by its instance index; the
      // late draw's first instance is set once the early phase has counted its own.
      args.extend_from_slice(&[StaticGeometry::CLUSTER_VERTICES, 0, 0, base]);
      late.extend_from_slice(&[StaticGeometry::CLUSTER_VERTICES, 0, 0, base]);
      base += capacity;
    }

    // The cull's counts, then the impostors' draw: a quad an impostor.
    args.extend_from_slice(&[0; 4]);
    args.extend_from_slice(&[6, 0, 0, 0]);
    late.extend_from_slice(&[0, 1, 1, 0]);

    self.list_capacity = base;
    self.lists.reserve(device, encoder, (base.max(1) as u64) * 8);
    self.candidates.reserve(device, encoder, (base.max(1) as u64) * 8);
    encoder.write_buffer(&self.regions, 0, bytemuck::cast_slice(&regions));
    encoder.write_buffer(&self.args_template, 0, bytemuck::cast_slice(&args));
    encoder.write_buffer(&self.late_template, 0, bytemuck::cast_slice(&late));
    self.written = Some(self.capacities);
  }
}
