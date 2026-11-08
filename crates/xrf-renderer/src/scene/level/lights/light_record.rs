use glam::{Mat4, Vec3, Vec4};
use xrf_renderer_core::ShaderStruct;
use xrf_visual::{LightDescription, LightKind};

use crate::lighting::lamp::light_basis::LightBasis;
use crate::lighting::lamp::light_bulb::get_bulb_radius;
use crate::lighting::lamp::light_shadow_size::to_light_shadow_scale;
use crate::scene::level::lights::asks::light_in_view::LightInView;
use crate::scene::texture::texture_cache::TextureCache;

/// One local light standing in view this frame, in view space, as `shaders/common/light_clusters.wgsl` declares it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
pub(crate) struct LightRecord {
  /// Its position, then `1 / L_R²`, the falloff reaching zero at 95% of the range.
  pub position: Vec4,
  /// Its colour, then its specular weight.
  pub color: Vec4,
  /// Where it points, then `cos` of half its cone, or `LightRecord::NO_CONE`.
  pub axis: Vec4,
  /// Its right, then its projection's scale, `cot` of half the widened cone.
  pub right: Vec4,
  /// Its up, then its projector's texture slot, or `LightRecord::NO_PROJECTOR`.
  pub up: Vec4,
  /// The sphere it is binned by.
  pub sphere: Vec4,
  /// A shadowed light's near and far planes and its face count, all zero for one unshadowed; then its bulb's radius,
  /// squared (`get_bulb_radius`).
  pub shadow: Vec4,
  /// Each face's square of the shadow atlas in texture coordinates, then how far the face has faded.
  pub faces: [Vec4; 6],
  /// What a metre of the air scatters of its light where all of it reaches, its shafts' as the view's engine draws
  /// them (`LightShafts::get_density`) for a shadowed light the level gives shafts, nought for any other; then nothing.
  pub scatter: Vec4,
}

impl LightRecord {
  /// What a spot without a projector writes as its slot.
  pub(crate) const NO_PROJECTOR: f32 = -1.0;

  /// What a light without a cone writes as its cone's cosine: every point passes its test.
  pub(crate) const NO_CONE: f32 = -2.0;

  /// A light's record, in view space: standing as `it` sees it, of `color` and `specular` weight, out to `range`, its
  /// projector's slot looked up in `projectors`. Unshadowed and unscattered until the frame fills those in.
  pub(crate) fn of(
    projectors: &[u32],
    light: &LightDescription,
    it: &LightInView,
    color: Vec3,
    specular: f32,
    range: f32,
    view: Mat4,
  ) -> Self {
    let (basis, bound): (&LightBasis, Vec4) = (&it.basis, it.bound);
    let is_spot: bool = matches!(light.kind, LightKind::Spot);
    let projector: f32 = light
      .projector
      .and_then(|projector| projectors.get(projector as usize))
      .filter(|slot| **slot != TextureCache::MISSING_SLOT)
      .map_or(Self::NO_PROJECTOR, |slot| *slot as f32);

    Self {
      position: view
        .transform_point3(basis.position)
        .extend(if range > 0.0 { 1.0 / (range * range) } else { 0.0 }),
      color: color.extend(specular),
      axis: view.transform_vector3(basis.direction).extend(if is_spot {
        (light.cone / 2.0).cos()
      } else {
        Self::NO_CONE
      }),
      right: view.transform_vector3(basis.right).extend(if is_spot {
        to_light_shadow_scale(light.cone)
      } else {
        0.0
      }),
      up: view
        .transform_vector3(basis.up)
        .extend(if is_spot { projector } else { Self::NO_PROJECTOR }),
      sphere: view.transform_point3(bound.truncate()).extend(bound.w),
      shadow: Vec4::new(0.0, 0.0, 0.0, get_bulb_radius(light).powi(2)),
      faces: [Vec4::ZERO; 6],
      scatter: Vec4::ZERO,
    }
  }
}
