use glam::Vec3;
use xrf_geometry::Ray;
use xrf_math::Vector2d;

use crate::light::light_point::LightPoint;
use crate::light::light_ray_group::LightRayGroup;
use crate::light::light_sample_point::LightSamplePoint;
use crate::shadow::shadow_texture::ShadowTexture;

/// A light's packet as `shadow_rays.wgsl` reads it (`Packet`): a directional light's, or a light at a position's.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuPacket {
  /// The direction toward a directional light, or a light's position.
  aim: [f32; 3],
  lift: f32,
  facing: [u32; LightRayGroup::FACING_WORDS],
  first_origin: u32,
  first_ray: u32,
  range: f32,
  kind: u32,
}

/// A point's rays' origin, the skip code of the face they pass, and the point itself (`Origin`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuOrigin {
  lifted: [f32; 3],
  skip: u32,
  point: [f32; 3],
  _pad: u32,
}

/// A single ray toward a light at a position, where its answer goes, and the face it passes (`Single`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuSingle {
  origin: [f32; 3],
  range: f32,
  direction: [f32; 3],
  index: u32,
  skip: u32,
  _pad: [u32; 3],
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

/// How many packets and single rays a batch holds, and the rules its rays follow (`Batch`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuBatchSize {
  pub(crate) packets: u32,
  pub(crate) singles: u32,
  pub(crate) rules: u32,
  _pad: u32,
}

impl GpuPacket {
  /// The kinds of packet, `KIND_*` in the shader.
  const KIND_DIRECTIONAL: u32 = 0;
  const KIND_LAMP: u32 = 1;

  /// A directional light's packet toward `toward`, its rays starting `lift` along it, from the points of `facing`
  /// whose origins start at `first_origin`, its answers from `first_ray`.
  pub(crate) fn new_directional(
    toward: Vec3,
    lift: f32,
    facing: [u32; LightRayGroup::FACING_WORDS],
    first_origin: u32,
    first_ray: u32,
  ) -> Self {
    Self {
      aim: toward.to_array(),
      lift,
      facing,
      first_origin,
      first_ray,
      range: LightPoint::DIRECTIONAL_RANGE,
      kind: Self::KIND_DIRECTIONAL,
    }
  }

  /// A light at `position`'s packet, from the points of `facing` whose origins start at `first_origin`, its answers
  /// from `first_ray`.
  pub(crate) fn new_lamp(
    position: Vec3,
    facing: [u32; LightRayGroup::FACING_WORDS],
    first_origin: u32,
    first_ray: u32,
  ) -> Self {
    Self {
      aim: position.to_array(),
      lift: 0.0,
      facing,
      first_origin,
      first_ray,
      range: 0.0,
      kind: Self::KIND_LAMP,
    }
  }
}

impl GpuOrigin {
  /// `point`'s origin, passing the face of skip code `skip`.
  pub(crate) fn new(point: &LightSamplePoint, skip: u32) -> Self {
    Self {
      lifted: point.lifted.to_array(),
      skip,
      point: point.point.to_array(),
      _pad: 0,
    }
  }
}

impl GpuSingle {
  /// `ray`, its answer at `index`, passing the face of skip code `skip` ([`crate::ray::gpu::ray_gpu_tracer::RayGpuTracer::get_skip`]).
  pub(crate) fn new(ray: &Ray, index: u32, skip: u32) -> Self {
    Self {
      origin: ray.origin.to_array(),
      range: ray.range,
      direction: ray.direction.to_array(),
      index,
      skip,
      _pad: [0; 3],
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
  pub(crate) fn new(packets: u32, singles: u32, rules: u32) -> Self {
    Self {
      packets,
      singles,
      rules,
      _pad: 0,
    }
  }
}
