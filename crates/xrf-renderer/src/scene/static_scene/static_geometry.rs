use glam::Vec4;
use xrf_error::XrfResult;
use xrf_renderer_core::{ProxyHandle, ProxyStore, Span, SpanBuffer, SpanLane, StoreMirror};
use xrf_visual::VisualClusters;

use crate::context::gpu_requirements::can_trace;
use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_static_report::RenderStaticReport;
use crate::scene::static_scene::static_cluster::StaticCluster;
use crate::scene::static_scene::static_impostor::StaticImpostor;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_place::StaticPlace;
use crate::scene::static_scene::static_row::StaticRow;
use crate::scene::static_scene::static_scene_writer::StaticSceneWriter;
use crate::scene::static_scene::static_slot::StaticSlot;
use crate::scene::static_scene::static_spans::StaticSpans;

/// A static scene's geometry on the GPU: span buffers of vertices, indices, clusters, slots, places, impostors, skins
/// and bones each item takes a range of, and the rows a store keeps dense for the cull to walk. A cull walks the
/// clusters and impostors whole, so a range is cleared as it is freed and reads as nothing.
pub struct StaticGeometry {
  /// Each layout's vertices, which an acceleration structure reads in place where rays are traced.
  pub words: [SpanBuffer; StaticLayout::COUNT],
  pub indices: SpanBuffer,
  /// The clusters, and their spheres in the `CLUSTER_SPHERES` lane.
  pub clusters: SpanBuffer,
  pub slots: SpanBuffer,
  pub places: SpanBuffer,
  /// Every place's rows, dense, for the row cull to walk.
  pub rows: StoreMirror,
  /// The impostors, and by lane their corners (two vectors a corner: its position and hemisphere term, then its atlas
  /// coordinate and sun term), what each one's level of detail draws this frame as the cull decided it, and the list of
  /// those drawn this frame.
  pub impostors: SpanBuffer,
  /// Skinned models' links, two words a vertex, and the objects' bone matrices, three rows a bone.
  pub skins: SpanBuffer,
  pub bones: SpanBuffer,
  row_records: ProxyStore<StaticRow>,
  /// Place zero, where every single draw stands: its geometry is in renderer space already. Never freed.
  origin: Span,
  /// Whether the device traces rays, so the vertices and indices are acceleration structures' input.
  is_traced: bool,
}

impl StaticGeometry {
  /// Triangles a cluster holds at most.
  pub const CLUSTER_TRIANGLES: u32 = VisualClusters::MAX_TRIANGLES;

  /// Vertices one cluster's draw pulls: its triangles' corners, a short cluster's tail collapsed.
  pub const CLUSTER_VERTICES: u32 = Self::CLUSTER_TRIANGLES * 3;

  /// The clusters' lane of their spheres.
  pub const CLUSTER_SPHERES: usize = 1;

  /// The impostors' lanes: their corners, their terms this frame, and the list of those drawn.
  pub const IMPOSTOR_CORNERS: usize = 1;
  pub const IMPOSTOR_TERMS: usize = 2;
  pub const IMPOSTOR_LIST: usize = 3;

  /// Vectors an impostor's corners take: two a corner.
  pub const CORNER_VECTORS: usize = StaticImpostor::CORNERS * 2;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let storage: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE;
    let words: u64 = u64::from(StaticLayout::STRIDE) * 4;
    let is_traced: bool = can_trace(device);
    let traced: wgpu::BufferUsages = if is_traced {
      storage | wgpu::BufferUsages::BLAS_INPUT
    } else {
      storage
    };
    let mut places: SpanBuffer = SpanBuffer::new(
      device,
      "static places",
      size_of::<StaticPlace>() as u64,
      storage,
      1 << 10,
    );
    let origin: Span = places
      .allocate(device, queue, 1)
      .expect("A fresh span buffer holds its first element");

    assert_eq!(origin.get_offset(), 0, "The origin place is the first");
    places.write(&origin, 0, bytemuck::bytes_of(&StaticPlace::default()));

    Self {
      words: [
        SpanBuffer::new(device, "static baked vertices", words, traced, 1 << 14),
        SpanBuffer::new(device, "static tree vertices", words, traced, 1 << 14),
        SpanBuffer::new(device, "static model vertices", words, traced, 1 << 14),
      ],
      indices: SpanBuffer::new(device, "static indices", 4, traced, 1 << 16),
      clusters: SpanBuffer::with_lanes(
        device,
        &[
          SpanLane::new("static clusters", size_of::<StaticCluster>() as u64),
          SpanLane::new("static spheres", size_of::<Vec4>() as u64),
        ],
        storage,
        1 << 12,
      ),
      slots: SpanBuffer::new(device, "static slots", size_of::<StaticSlot>() as u64, storage, 1 << 10),
      places,
      rows: StoreMirror::new(device, "static rows", size_of::<StaticRow>() as u64, storage),
      impostors: SpanBuffer::with_lanes(
        device,
        &[
          SpanLane::new("static impostors", size_of::<StaticImpostor>() as u64),
          SpanLane::new(
            "static impostor corners",
            (Self::CORNER_VECTORS * size_of::<Vec4>()) as u64,
          ),
          SpanLane::new("static impostor terms", 16),
          SpanLane::new("static impostor list", 4),
        ],
        storage,
        1 << 8,
      ),
      skins: SpanBuffer::new(device, "static model skins", 8, storage, 1 << 14),
      bones: SpanBuffer::new(device, "static bones", size_of::<Vec4>() as u64, storage, 1 << 12),
      row_records: ProxyStore::new(),
      origin,
      is_traced,
    }
  }

  /// How full its pools are, as a report of the static draws has them.
  pub fn get_pools(&self) -> RenderStaticReport {
    RenderStaticReport {
      slots: to_pool_use(&self.slots),
      places: to_pool_use(&self.places),
      rows: RenderPoolUse {
        used: self.row_records.len() as u32,
        capacity: self.rows.get_capacity(),
      },
      lods: to_pool_use(&self.impostors),
      clusters: to_pool_use(&self.clusters),
      ..Default::default()
    }
  }

  /// Clusters the cull walks: the whole buffer, a range nothing holds reading as an empty cluster.
  pub fn get_cluster_count(&self) -> u32 {
    self.clusters.get_capacity()
  }

  /// Rows the cull walks, as of the last `flush`.
  pub fn get_row_count(&self) -> u32 {
    self.rows.get_count()
  }

  /// Impostors the cull walks: the whole buffer, a range nothing holds reading as no impostor.
  pub fn get_impostor_count(&self) -> u32 {
    self.impostors.get_capacity()
  }

  /// Place zero, where every single draw stands.
  pub fn get_origin(&self) -> u32 {
    self.origin.get_offset()
  }

  /// Whether the device traces rays, so the vertices and indices are acceleration structures' input.
  pub fn is_traced(&self) -> bool {
    self.is_traced
  }

  /// Uploads what was added, changed and removed since the last frame.
  pub fn flush(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    for buffer in self.list_spans_mut() {
      buffer.flush(queue);
    }

    self.rows.sync(device, queue, &mut self.row_records, |row| *row);
  }

  pub fn add_row(&mut self, row: StaticRow) -> ProxyHandle<StaticRow> {
    self.row_records.add(row)
  }

  pub fn remove_row(&mut self, row: ProxyHandle<StaticRow>) {
    self.row_records.remove(row);
  }

  /// Takes a range of each buffer for what a writer gathered, none where it gathered nothing; on an error, what was
  /// taken is given back.
  pub fn allocate(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    writer: &StaticSceneWriter,
  ) -> XrfResult<StaticSpans> {
    let mut spans: StaticSpans = StaticSpans::default();
    let counts: [usize; 6] = [
      writer.indices.len(),
      writer.clusters.len(),
      writer.slots.len(),
      writer.places.len(),
      writer.impostors.len(),
      writer.skins.len() / 2,
    ];
    let mut result: XrfResult<()> = Ok(());

    for (layout, words) in writer.words.iter().enumerate() {
      if result.is_ok() {
        result = allocate_span(
          &mut self.words[layout],
          (device, queue),
          words.len() / StaticLayout::STRIDE as usize,
        )
        .map(|span| spans.words[layout] = span);
      }
    }

    let buffers: [(&mut SpanBuffer, &mut Option<Span>); 6] = [
      (&mut self.indices, &mut spans.indices),
      (&mut self.clusters, &mut spans.clusters),
      (&mut self.slots, &mut spans.slots),
      (&mut self.places, &mut spans.places),
      (&mut self.impostors, &mut spans.impostors),
      (&mut self.skins, &mut spans.skins),
    ];

    for ((buffer, held), count) in buffers.into_iter().zip(counts) {
      if result.is_ok() {
        result = allocate_span(buffer, (device, queue), count).map(|span| *held = span);
      }
    }

    match result {
      Ok(()) => Ok(spans),
      Err(error) => {
        self.release(spans);

        Err(error)
      }
    }
  }

  /// Queues a writer's records for the ranges taken for them.
  pub fn write(&mut self, spans: &StaticSpans, writer: &StaticSceneWriter) {
    for (layout, words) in writer.words.iter().enumerate() {
      if let Some(span) = &spans.words[layout] {
        self.words[layout].write(span, 0, bytemuck::cast_slice(words));
      }
    }

    if let Some(span) = &spans.indices {
      self.indices.write(span, 0, bytemuck::cast_slice(&writer.indices));
    }

    if let Some(span) = &spans.clusters {
      self.clusters.write(span, 0, bytemuck::cast_slice(&writer.clusters));
      self
        .clusters
        .write_lane(span, Self::CLUSTER_SPHERES, 0, bytemuck::cast_slice(&writer.spheres));
    }

    if let Some(span) = &spans.slots {
      self.slots.write(span, 0, bytemuck::cast_slice(&writer.slots));
    }

    if let Some(span) = &spans.places {
      self.places.write(span, 0, bytemuck::cast_slice(&writer.places));
    }

    if let Some(span) = &spans.impostors {
      self.impostors.write(span, 0, bytemuck::cast_slice(&writer.impostors));
      self
        .impostors
        .write_lane(span, Self::IMPOSTOR_CORNERS, 0, bytemuck::cast_slice(&writer.corners));
    }

    if let Some(span) = &spans.skins {
      self.skins.write(span, 0, bytemuck::cast_slice(&writer.skins));
    }
  }

  /// Gives an item's ranges back, the clusters and impostors cleared first, since the cull walks them whole.
  pub fn release(&mut self, spans: StaticSpans) {
    let StaticSpans {
      words,
      indices,
      clusters,
      slots,
      places,
      impostors,
      skins,
    } = spans;

    for (layout, span) in words.into_iter().enumerate() {
      if let Some(span) = span {
        self.words[layout].free(span);
      }
    }

    if let Some(span) = clusters {
      self.clusters.clear(&span);
      self.clusters.free(span);
    }

    if let Some(span) = impostors {
      self.impostors.clear(&span);
      self.impostors.free(span);
    }

    for (buffer, span) in [
      (&mut self.indices, indices),
      (&mut self.slots, slots),
      (&mut self.places, places),
      (&mut self.skins, skins),
    ] {
      if let Some(span) = span {
        buffer.free(span);
      }
    }
  }

  /// Every span buffer, in the order their bytes and generations are summed.
  fn list_spans(&self) -> [&SpanBuffer; 10] {
    [
      &self.words[0],
      &self.words[1],
      &self.words[2],
      &self.indices,
      &self.clusters,
      &self.slots,
      &self.places,
      &self.impostors,
      &self.skins,
      &self.bones,
    ]
  }

  fn list_spans_mut(&mut self) -> [&mut SpanBuffer; 10] {
    let [baked, tree, model] = &mut self.words;

    [
      baked,
      tree,
      model,
      &mut self.indices,
      &mut self.clusters,
      &mut self.slots,
      &mut self.places,
      &mut self.impostors,
      &mut self.skins,
      &mut self.bones,
    ]
  }
}

/// A range of `count` elements of a buffer, none for no elements.
fn allocate_span(
  buffer: &mut SpanBuffer,
  (device, queue): (&wgpu::Device, &wgpu::Queue),
  count: usize,
) -> XrfResult<Option<Span>> {
  if count == 0 {
    return Ok(None);
  }

  buffer.allocate(device, queue, count as u32).map(Some)
}

/// How full a span buffer is: its elements held, and its capacity.
fn to_pool_use(buffer: &SpanBuffer) -> RenderPoolUse {
  RenderPoolUse {
    used: buffer.get_capacity() - buffer.get_free(),
    capacity: buffer.get_capacity(),
  }
}
