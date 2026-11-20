use glam::Vec3;

use crate::ray::ray_group::RayGroup;

/// A light's packet as `shadow_rays.wgsl` reads it (`Packet`): a directional light's, or a light at a position's.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct RayGpuPacket {
  /// The direction toward a directional light, or a light's position.
  aim: [f32; 3],
  lift: f32,
  facing: [u32; RayGroup::FACING_WORDS],
  first_origin: u32,
  first_ray: u32,
  range: f32,
  kind: u32,
}

impl RayGpuPacket {
  /// The kinds of packet, `KIND_*` in the shader.
  const KIND_DIRECTIONAL: u32 = 0;
  const KIND_LAMP: u32 = 1;

  /// A directional light's packet toward `toward` for `range`, its rays starting `lift` along it, from the points of
  /// `facing` whose origins start at `first_origin`, its answers from `first_ray`.
  pub(crate) fn new_directional(
    toward: Vec3,
    lift: f32,
    range: f32,
    facing: [u32; RayGroup::FACING_WORDS],
    first_origin: u32,
    first_ray: u32,
  ) -> Self {
    Self {
      aim: toward.to_array(),
      lift,
      facing,
      first_origin,
      first_ray,
      range,
      kind: Self::KIND_DIRECTIONAL,
    }
  }

  /// A light at `position`'s packet, from the points of `facing` whose origins start at `first_origin`, its answers
  /// from `first_ray`.
  pub(crate) fn new_lamp(
    position: Vec3,
    facing: [u32; RayGroup::FACING_WORDS],
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
