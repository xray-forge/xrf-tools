use crate::pass::grass_pass::GrassPass;
use crate::scene::level::grass_build_size::{GRASS_ITEM_BYTES, GrassBuildSize};

/// Words a cached slot's key takes.
const KEY_WORDS: u64 = 4;

/// A ring of planted slots and item lists of one size, `CDetailManager`'s cache on the GPU: made zeroed, every key
/// holding generation nought, which no planting runs under, so every cell starts stale.
pub struct GrassBuild {
  pub size: GrassBuildSize,
  /// The items sorted by model, which the draws read.
  pub sorted: wgpu::Buffer,
  pub bind_group: wgpu::BindGroup,
}

impl GrassBuild {
  pub fn new(device: &wgpu::Device, pass: &GrassPass, size: GrassBuildSize) -> Self {
    let buffer = |label: &str, bytes: u64| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.max(4).next_multiple_of(4),
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
      })
    };
    let cells: u64 = u64::from(size.cells);
    let buffers: [wgpu::Buffer; 8] = [
      buffer("grass keys", cells * KEY_WORDS * 4),
      buffer("grass cell shapes", cells * 16),
      buffer(
        "grass cached tufts",
        cells * u64::from(size.per_cell) * GRASS_ITEM_BYTES,
      ),
      buffer("grass band counts", u64::from(size.bands) * 4),
      buffer("grass schedule", 12),
      buffer("grass planted", u64::from(size.capacity) * GRASS_ITEM_BYTES),
      buffer("grass planted models", u64::from(size.capacity) * 4),
      buffer("grass sorted", u64::from(size.capacity) * GRASS_ITEM_BYTES),
    ];
    let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("grass build"),
      layout: pass.get_build_layout(),
      entries: &buffers
        .iter()
        .enumerate()
        .map(|(binding, buffer)| wgpu::BindGroupEntry {
          binding: binding as u32,
          resource: buffer.as_entire_binding(),
        })
        .collect::<Vec<_>>(),
    });
    let [_, _, _, _, _, _, _, sorted] = buffers;

    Self {
      size,
      sorted,
      bind_group,
    }
  }
}
