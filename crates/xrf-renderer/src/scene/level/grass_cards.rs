use crate::contract::render_grass_settings::RenderGrassSettings;

/// The grass that casts a view's shadows (`shaders/grass/cards.wgsl`): the tufts near its eye stood and swayed as the
/// frame draws them each frame, their triangles' corners and records and how many stand; the structure built over them,
/// which the view's own top level holds beside the scene's where it traces its light, and the draw of them into the
/// near sun cascades where those shadow the frame (`shaders/grass/card_shadow.wgsl`).
pub struct GrassCards {
  pub vertices: wgpu::Buffer,
  pub triangles: wgpu::Buffer,
  pub count: wgpu::Buffer,
  /// The cascades' indirect draw of the corners that stand.
  pub draw: wgpu::Buffer,
  /// Triangles they hold at most (`get_capacity`).
  pub capacity: u32,
  pub blas: wgpu::Blas,
}

impl GrassCards {
  /// The least and the most triangles the cards hold, and how much more room than their tufts asked they are given
  /// (`get_capacity`).
  const ASKED_MARGIN: f32 = 1.25;
  const LEAST_CAPACITY: u32 = 1 << 14;
  const MOST_CAPACITY: u32 = 1 << 21;
  /// Metres the cards reach at most.
  const MOST_REACH: f32 = 150.0;
  /// Bytes a triangle's corners and its record take.
  const VERTEX_BYTES: u64 = 36;
  const TRIANGLE_BYTES: u64 = 16;
  /// The top-level instance's custom index (`SCENE_GRASS_CARDS`), past any of the scene's, and its mask, which a ray
  /// leaves out by its own (`SCENE_MASK_SCENE`).
  pub const INSTANCE: u32 = 0x00ff_ffff;
  pub const MASK: u8 = 0x02;
  /// The share of their reach over which the cards fade out.
  const FADE_SHARE: f32 = 0.2;

  /// Cards holding `capacity` triangles at most.
  pub fn new(device: &wgpu::Device, capacity: u32) -> Self {
    let buffer = |label: &str, size: u64, usage: wgpu::BufferUsages| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::STORAGE | usage,
        mapped_at_creation: false,
      })
    };
    let triangles_most: u64 = u64::from(capacity);
    let vertices: wgpu::Buffer = buffer(
      "grass card vertices",
      triangles_most * Self::VERTEX_BYTES,
      wgpu::BufferUsages::BLAS_INPUT,
    );
    let triangles: wgpu::Buffer = buffer(
      "grass card triangles",
      triangles_most * Self::TRIANGLE_BYTES,
      wgpu::BufferUsages::empty(),
    );
    // Four words, as a stats readback copies them, the first counted.
    let count: wgpu::Buffer = buffer("grass card count", 16, wgpu::BufferUsages::COPY_SRC);
    let draw: wgpu::Buffer = buffer("grass card draw", 16, wgpu::BufferUsages::INDIRECT);
    let blas: wgpu::Blas = device.create_blas(
      &wgpu::CreateBlasDescriptor {
        label: Some("grass cards"),
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_BUILD,
        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
      },
      wgpu::BlasGeometrySizeDescriptors::Triangles {
        descriptors: vec![Self::get_size(capacity)],
      },
    );
    Self {
      vertices,
      triangles,
      count,
      draw,
      capacity,
      blas,
    }
  }

  /// Triangles the cards hold, `current` held now and their tufts having last asked for `asked`: `ASKED_MARGIN` more than
  /// asked, as a power of two; grown at once and shrunk only once that falls to half the room.
  pub fn get_capacity(current: u32, asked: u32) -> u32 {
    let wanted: u32 = ((asked as f32 * Self::ASKED_MARGIN) as u32)
      .next_power_of_two()
      .clamp(Self::LEAST_CAPACITY, Self::MOST_CAPACITY);

    if wanted > current || wanted.saturating_mul(2) <= current {
      wanted
    } else {
      current
    }
  }

  /// Metres from the eye within which the cards stand a tuft for `settings`, no further than `most` where it is given,
  /// and the last of them over which they fade.
  pub fn get_reach(settings: &RenderGrassSettings, most: Option<f32>) -> (f32, f32) {
    let reach: f32 = most
      .map_or(settings.shadow_radius, |most| settings.shadow_radius.min(most))
      .clamp(1.0, Self::MOST_REACH);

    (reach, reach * Self::FADE_SHARE)
  }

  /// Slots each way from the eye's that `reach` spans: the eye stands up to half a slot off its slot's middle.
  pub fn get_slots(reach: f32) -> u32 {
    (reach / 2.0) as u32 + 2
  }

  /// Ring cells about the eye's whose tufts may stand within `slots` of it.
  pub fn get_window_cells(slots: u32) -> u32 {
    (slots * 2 + 1) * (slots * 2 + 1)
  }

  /// Every triangle of `capacity` the cards hold, those not standing this frame marked as none by their corners.
  pub fn get_size(capacity: u32) -> wgpu::BlasTriangleGeometrySizeDescriptor {
    wgpu::BlasTriangleGeometrySizeDescriptor {
      vertex_format: wgpu::VertexFormat::Float32x3,
      vertex_count: capacity * 3,
      index_format: None,
      index_count: None,
      flags: wgpu::AccelerationStructureGeometryFlags::empty(),
    }
  }
}

impl GrassCards {
  /// Bytes its corners, records, count and draw take, its structure apart.
  pub fn get_memory(&self) -> u64 {
    [&self.vertices, &self.triangles, &self.count, &self.draw]
      .into_iter()
      .map(wgpu::Buffer::size)
      .sum()
  }
}
