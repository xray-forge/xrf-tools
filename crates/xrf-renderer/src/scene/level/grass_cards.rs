use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::pass::grass_pass::GrassPass;
use crate::scene::level::grass_level::GrassLevel;

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
  pub blas: wgpu::Blas,
  /// What standing them binds at group 2, and what their draw into a cascade binds at group 1.
  pub group: wgpu::BindGroup,
  pub shadow_group: wgpu::BindGroup,
}

impl GrassCards {
  /// Triangles the cards hold at most, as `shaders/grass/cards.wgsl` stands them within the widest reach: some 800
  /// stand within 12 metres of the marsh.
  pub const CAPACITY: u32 = 1 << 16;
  /// Bytes a triangle's corners and its record take.
  const VERTEX_BYTES: u64 = 36;
  const TRIANGLE_BYTES: u64 = 16;
  /// The top-level instance's custom index (`SCENE_GRASS_CARDS`), past any of the scene's, and its mask, which a ray
  /// leaves out by its own (`SCENE_MASK_SCENE`).
  pub const INSTANCE: u32 = 0x00ff_ffff;
  pub const MASK: u8 = 0x02;
  /// The share of their reach over which the cards fade out.
  const FADE_SHARE: f32 = 0.2;

  /// Cards for `level`'s grass, swayed by `wind`.
  pub fn new(device: &wgpu::Device, pass: &GrassPass, level: &GrassLevel, wind: &wgpu::Buffer) -> Self {
    let buffer = |label: &str, size: u64, usage: wgpu::BufferUsages| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::STORAGE | usage,
        mapped_at_creation: false,
      })
    };
    let capacity: u64 = u64::from(Self::CAPACITY);
    let vertices: wgpu::Buffer = buffer(
      "grass card vertices",
      capacity * Self::VERTEX_BYTES,
      wgpu::BufferUsages::BLAS_INPUT,
    );
    let triangles: wgpu::Buffer = buffer(
      "grass card triangles",
      capacity * Self::TRIANGLE_BYTES,
      wgpu::BufferUsages::empty(),
    );
    let count: wgpu::Buffer = buffer("grass card count", 4, wgpu::BufferUsages::empty());
    let draw: wgpu::Buffer = buffer("grass card draw", 16, wgpu::BufferUsages::INDIRECT);
    let blas: wgpu::Blas = device.create_blas(
      &wgpu::CreateBlasDescriptor {
        label: Some("grass cards"),
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_BUILD,
        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
      },
      wgpu::BlasGeometrySizeDescriptors::Triangles {
        descriptors: vec![Self::get_size()],
      },
    );
    let group: wgpu::BindGroup = pass.create_cards_group(
      device,
      (&level.positions, &level.uvs, &level.indices),
      wind,
      (&vertices, &triangles, &count, &draw),
    );
    let shadow_group: wgpu::BindGroup = pass.create_card_shadow_group(device, (&vertices, &triangles), &level.models);

    Self {
      vertices,
      triangles,
      count,
      draw,
      blas,
      group,
      shadow_group,
    }
  }

  /// Metres from the eye within which the cards stand a tuft for `settings`, and the last of them over which they fade.
  pub fn get_reach(settings: &RenderGrassSettings) -> (f32, f32) {
    let reach: f32 = settings.shadow_radius.clamp(1.0, 64.0);

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

  /// Every triangle the cards hold, those not standing this frame marked as none by their corners.
  pub fn get_size() -> wgpu::BlasTriangleGeometrySizeDescriptor {
    wgpu::BlasTriangleGeometrySizeDescriptor {
      vertex_format: wgpu::VertexFormat::Float32x3,
      vertex_count: Self::CAPACITY * 3,
      index_format: None,
      index_count: None,
      flags: wgpu::AccelerationStructureGeometryFlags::empty(),
    }
  }
}
