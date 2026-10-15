use crate::pass::grass_pass::GrassPass;
use crate::scene::level::grass_build_size::{GRASS_ITEM_BYTES, GrassBuildSize};

/// Words a cached slot's key takes.
const KEY_WORDS: u64 = 4;

/// A ring of planted slots and item lists of one size, `CDetailManager`'s cache on the GPU: made zeroed, every key
/// holding generation nought, which no planting runs under, so every cell starts stale.
pub struct GrassBuild {
  pub size: GrassBuildSize,
  pub bind_group: wgpu::BindGroup,
  /// The puddles the group binds.
  puddles: wgpu::TextureView,
  /// The ring, the bands' counts, the schedule and the lists, as the group binds them.
  buffers: [wgpu::Buffer; 8],
  /// The sort's indirect dispatch, and the group `arrange` writes it through.
  pub sort_dispatch: wgpu::Buffer,
  pub sort_group: wgpu::BindGroup,
}

impl GrassBuild {
  /// A ring of `size`, planted with the view's parameters in `uniform`.
  /// `puddles` are this frame's puddle sites and where they stand, which the planting plants no tuft in.
  pub fn new(
    device: &wgpu::Device,
    pass: &GrassPass,
    size: GrassBuildSize,
    uniform: &wgpu::Buffer,
    puddles: (&wgpu::TextureView, &wgpu::Buffer),
  ) -> Self {
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
    let bind_group: wgpu::BindGroup = Self::create_group(device, pass, &buffers, uniform, puddles);
    let sort_dispatch: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("grass sort dispatch"),
      size: 12,
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
      mapped_at_creation: false,
    });
    let sort_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("grass sort"),
      layout: pass.get_sort_layout(),
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: sort_dispatch.as_entire_binding(),
      }],
    });

    Self {
      size,
      bind_group,
      puddles: puddles.0.clone(),
      buffers,
      sort_dispatch,
      sort_group,
    }
  }

  /// The items sorted by model, which the draws read.
  pub fn get_sorted(&self) -> &wgpu::Buffer {
    &self.buffers[7]
  }

  /// Binds this frame's puddles where they are not the ones bound: the level's surface is made after the ring, and
  /// made again as the camera leaves it.
  pub fn bind_puddles(
    &mut self,
    device: &wgpu::Device,
    pass: &GrassPass,
    uniform: &wgpu::Buffer,
    puddles: (&wgpu::TextureView, &wgpu::Buffer),
  ) {
    if *puddles.0 != self.puddles {
      self.bind_group = Self::create_group(device, pass, &self.buffers, uniform, puddles);
      self.puddles = puddles.0.clone();
    }
  }

  fn create_group(
    device: &wgpu::Device,
    pass: &GrassPass,
    buffers: &[wgpu::Buffer; 8],
    uniform: &wgpu::Buffer,
    puddles: (&wgpu::TextureView, &wgpu::Buffer),
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("grass build"),
      layout: pass.get_build_layout(),
      entries: &buffers
        .iter()
        .chain([uniform])
        .enumerate()
        .map(|(binding, buffer)| wgpu::BindGroupEntry {
          binding: binding as u32,
          resource: buffer.as_entire_binding(),
        })
        .chain([
          wgpu::BindGroupEntry {
            binding: 9,
            resource: wgpu::BindingResource::TextureView(puddles.0),
          },
          wgpu::BindGroupEntry {
            binding: 10,
            resource: puddles.1.as_entire_binding(),
          },
        ])
        .collect::<Vec<_>>(),
    })
  }
}

impl GrassBuild {
  /// Bytes its ring's and lists' buffers take.
  pub fn get_memory(&self) -> u64 {
    self
      .buffers
      .iter()
      .chain([&self.sort_dispatch])
      .map(wgpu::Buffer::size)
      .sum()
  }
}
