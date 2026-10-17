use crate::scene::level::grass_build_size::GrassBuildSize;

/// Words a cached slot's key takes.
const KEY_WORDS: u64 = 4;

/// A ring of planted slots and item lists of one size, `CDetailManager`'s cache on the GPU: made zeroed, every key
/// holding generation nought, which no planting runs under, so every cell starts stale.
pub struct GrassBuild {
  pub size: GrassBuildSize,
  /// The ring: a key a cell, a vector a cell (its ground's middle height and half its height, its hemisphere and sun),
  /// and two vectors a cached tuft.
  pub keys: wgpu::Buffer,
  pub shapes: wgpu::Buffer,
  pub cached: wgpu::Buffer,
  /// Stale cells a band counts, then the schedule: the band the frame's planting stops in, what it plants there,
  /// tickets.
  pub band_counts: wgpu::Buffer,
  pub schedule: wgpu::Buffer,
  /// The frame's items as planted, each one's model, and the same sorted by model, which the draws read.
  pub planted: wgpu::Buffer,
  pub planted_models: wgpu::Buffer,
  pub sorted: wgpu::Buffer,
  /// The sort's indirect dispatch, which `arrange` writes.
  pub sort_dispatch: wgpu::Buffer,
}

impl GrassBuild {
  /// A ring and lists of `size`.
  pub fn new(device: &wgpu::Device, size: GrassBuildSize) -> Self {
    let buffer = |label: &str, bytes: u64| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.max(4).next_multiple_of(4),
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
      })
    };
    let cells: u64 = u64::from(size.cells);

    Self {
      size,
      keys: buffer("grass keys", cells * KEY_WORDS * 4),
      shapes: buffer("grass cell shapes", cells * 16),
      cached: buffer(
        "grass cached tufts",
        cells * u64::from(size.per_cell) * GrassBuildSize::ITEM_BYTES,
      ),
      band_counts: buffer("grass band counts", u64::from(size.bands) * 4),
      schedule: buffer("grass schedule", 12),
      planted: buffer("grass planted", u64::from(size.capacity) * GrassBuildSize::ITEM_BYTES),
      planted_models: buffer("grass planted models", u64::from(size.capacity) * 4),
      sorted: buffer("grass sorted", u64::from(size.capacity) * GrassBuildSize::ITEM_BYTES),
      sort_dispatch: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("grass sort dispatch"),
        size: 12,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
        mapped_at_creation: false,
      }),
    }
  }
}

impl GrassBuild {
  /// Bytes its ring's and lists' buffers take.
  pub fn get_memory(&self) -> u64 {
    [
      &self.keys,
      &self.shapes,
      &self.cached,
      &self.band_counts,
      &self.schedule,
      &self.planted,
      &self.planted_models,
      &self.sorted,
      &self.sort_dispatch,
    ]
    .into_iter()
    .map(wgpu::Buffer::size)
    .sum()
  }
}
