use xrf_error::{XrfError, XrfResult};

use crate::frame::frame_encoder::FrameEncoder;
use crate::scene::static_scene::draw::static_batch::StaticBatch;
use crate::scene::static_scene::draw::static_region::StaticRegion;
use crate::scene::static_scene::geometry::static_geometry::StaticGeometry;

/// What every cull of a static scene starts from: each draw list's run of a visible list, and the draw arguments' and
/// the late arguments' templates, which a cull's own are reset from before it; sized by the entries each draw list may
/// need. Each cull writes lists and arguments of its own: a view's camera (`StaticViewCull`), a shadow's (`ShadowCut`),
/// a light's shadow faces.
pub struct StaticCullScratch {
  pub regions: wgpu::Buffer,
  /// Every batch's draw arguments as a cull starts them, copied into a cull's own before it.
  pub args_template: wgpu::Buffer,
  /// The late draw arguments as a camera's cull starts them, copied into its own before it.
  pub late_template: wgpu::Buffer,
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

  /// Bytes the late draw arguments take, the late phase's dispatch and the candidates' count with them.
  pub const LATE_SIZE: u64 = Self::LATE_DISPATCH_OFFSET + 16;

  /// Bytes an entry of the visible list takes.
  pub const ENTRY_BYTES: u64 = 8;

  /// The most copies of the visible list one frame's buffer holds: a light's shadow faces drawn in a frame.
  pub const MOST_LIST_COPIES: u64 = 8;

  pub fn new(device: &wgpu::Device) -> Self {
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
      regions: buffer(
        "static regions",
        (StaticBatch::LIST_COUNT * size_of::<StaticRegion>()) as u64,
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      ),
      args_template: buffer("static draw arguments template", Self::ARGS_SIZE, template),
      late_template: buffer("static late draw arguments template", Self::LATE_SIZE, template),
      capacities: [0; StaticBatch::LIST_COUNT],
      written: None,
      list_capacity: 0,
    }
  }

  /// Refuses an item whose clusters would grow the visible list, at the most copies of it one buffer holds, past what
  /// the device binds as one storage buffer; asked before the item allocates anything.
  pub fn check_capacities(&self, device: &wgpu::Device, added: &[u32; StaticBatch::LIST_COUNT]) -> XrfResult {
    let limits: wgpu::Limits = device.limits();
    let entries: u64 = self
      .capacities
      .iter()
      .zip(added)
      .map(|(total, added)| u64::from(*total) + u64::from(*added))
      .sum();
    let region: u64 =
      (entries.max(1) * Self::ENTRY_BYTES).next_multiple_of(u64::from(limits.min_storage_buffer_offset_alignment));
    let bytes: u64 = region * Self::MOST_LIST_COPIES;
    let most: u64 = limits.max_buffer_size.min(limits.max_storage_buffer_binding_size);

    if bytes > most {
      return Err(XrfError::new_unexpected_error(format!(
        "The static scene's visible list would take {bytes} bytes, past the {most} one GPU buffer holds"
      )));
    }

    Ok(())
  }

  /// Adds an item's clusters to each draw list's capacity, which sizes the lists' regions.
  pub fn add_capacities(&mut self, capacities: &[u32; StaticBatch::LIST_COUNT]) {
    for (total, added) in self.capacities.iter_mut().zip(capacities) {
      *total += added;
    }
  }

  /// Takes an item's clusters back off each draw list's capacity.
  pub fn remove_capacities(&mut self, capacities: &[u32; StaticBatch::LIST_COUNT]) {
    for (total, removed) in self.capacities.iter_mut().zip(capacities) {
      *total -= removed;
    }
  }

  /// Writes the runs of the visible list and the templates the culls' arguments are reset from, where the capacities
  /// changed since they were last written.
  pub fn prepare(&mut self, encoder: &mut FrameEncoder) {
    if self.written != Some(self.capacities) {
      self.write(encoder);
    }
  }

  /// Resets a cull's draw arguments empty, as every draw list's start a frame.
  pub fn reset_args(&self, encoder: &mut wgpu::CommandEncoder, args: &wgpu::Buffer) {
    encoder.copy_buffer_to_buffer(&self.args_template, 0, args, 0, Self::ARGS_SIZE);
  }

  /// Entries the visible list holds, and the candidates as many.
  pub fn get_list_capacity(&self) -> u32 {
    self.list_capacity
  }

  /// Bytes one copy of the visible list takes, at least an entry's, which every cull's lists are made at.
  pub fn get_list_bytes(&self) -> u64 {
    u64::from(self.list_capacity.max(1)) * Self::ENTRY_BYTES
  }

  /// Writes each draw list's run of the visible list and the templates its draw arguments are reset from.
  fn write(&mut self, encoder: &mut FrameEncoder) {
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
    encoder.write_buffer(&self.regions, 0, bytemuck::cast_slice(&regions));
    encoder.write_buffer(&self.args_template, 0, bytemuck::cast_slice(&args));
    encoder.write_buffer(&self.late_template, 0, bytemuck::cast_slice(&late));
    self.written = Some(self.capacities);
  }
}
