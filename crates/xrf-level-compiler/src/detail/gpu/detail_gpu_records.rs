use glam::Vec3;
use xrf_geometry::Ray;
use xrf_math::Vector2d;

use crate::detail::detail_slot_plan::DetailRayGroup;
use crate::shadow::shadow_texture::ShadowTexture;

/// A directional light's packet as `detail_shadow_rays.wgsl` reads it (`Packet`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuPacket {
  toward: [f32; 3],
  lift: f32,
  facing: [u32; DetailRayGroup::FACING_WORDS],
  first_origin: u32,
  first_ray: u32,
  range: f32,
  _pad: u32,
}

/// A single ray toward a light at a position, and where its answer goes (`Single`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuSingle {
  origin: [f32; 3],
  range: f32,
  direction: [f32; 3],
  index: u32,
}

/// A face dimming light through alpha, as the shader reads it (`AlphaFace`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuAlphaFace {
  corners: [[f32; 2]; 3],
  texture: u32,
  _pad: u32,
}

/// Where an alpha texture's texels lie among the uploaded bytes, and its size (`AlphaTexture`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuAlphaTexture {
  offset: u32,
  width: u32,
  height: u32,
  _pad: u32,
}

/// How many packets and single rays a batch holds (`Batch`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuBatchSize {
  pub(crate) packets: u32,
  pub(crate) singles: u32,
  _pad: [u32; 2],
}

impl GpuPacket {
  /// A packet toward `toward`, its rays starting `lift` along it, from the points of `facing` whose origins start at
  /// `first_origin`, its answers from `first_ray`.
  pub(crate) fn new(
    toward: Vec3,
    lift: f32,
    facing: [u32; DetailRayGroup::FACING_WORDS],
    first_origin: u32,
    first_ray: u32,
  ) -> Self {
    Self {
      toward: toward.to_array(),
      lift,
      facing,
      first_origin,
      first_ray,
      range: DetailRayGroup::DIRECTIONAL_RANGE,
      _pad: 0,
    }
  }
}

impl GpuSingle {
  /// `ray`, its answer at `index`.
  pub(crate) fn new(ray: &Ray, index: u32) -> Self {
    Self {
      origin: ray.origin.to_array(),
      range: ray.range,
      direction: ray.direction.to_array(),
      index,
    }
  }
}

impl GpuAlphaFace {
  /// A face of texture `texture` with corners at `coordinates`, as `ShadowMaterials::get_alpha_faces` gives it.
  pub(crate) fn of((texture, coordinates): (u32, [Vector2d; 3])) -> Self {
    Self {
      corners: coordinates.map(|corner| [corner.x, corner.y]),
      texture,
      _pad: 0,
    }
  }
}

impl GpuAlphaTexture {
  /// Each of `textures`' place among the bytes, and every texel's byte, the textures one after another, padded to a
  /// whole word.
  pub(crate) fn pack(textures: &[ShadowTexture]) -> (Vec<Self>, Vec<u8>) {
    let mut bytes: Vec<u8> = Vec::new();
    let records: Vec<Self> = textures
      .iter()
      .map(|texture| {
        let record: Self = Self {
          offset: bytes.len() as u32,
          width: texture.alpha.width(),
          height: texture.alpha.height(),
          _pad: 0,
        };

        bytes.extend_from_slice(texture.alpha.as_raw());

        record
      })
      .collect();

    bytes.resize(bytes.len().next_multiple_of(4), 0);

    (records, bytes)
  }
}

impl GpuBatchSize {
  pub(crate) fn new(packets: u32, singles: u32) -> Self {
    Self {
      packets,
      singles,
      _pad: [0; 2],
    }
  }
}
