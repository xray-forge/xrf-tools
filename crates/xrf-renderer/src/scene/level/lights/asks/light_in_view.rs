use glam::{Vec3, Vec4};
use xrf_renderer_core::ProxyHandle;
use xrf_visual::{LightDescription, LightKind};

use crate::lighting::lamp::light_basis::LightBasis;
use crate::lighting::lamp::light_shadow_size::LIGHT_SHADOW_POINT_FACES;

/// A light standing in a view this frame.
pub(crate) struct LightInView {
  /// Metres from the eye to its bound's edge, which the nearest are kept by.
  pub distance: f32,
  /// Whether the view sees its bound; a shadowed light out of view near the eye is only kept ready.
  pub is_seen: bool,
  pub handle: ProxyHandle<LightDescription>,
  pub basis: LightBasis,
  pub bound: Vec4,
  pub fades: LightFades,
  /// How much of it its campfire lets show, all of it for any other light.
  pub share: f32,
}

/// How far a light has faded: whole, as a shadowed spot fades, and each face, as a shadowed point's omni parts each
/// fade on their own, the most any of it shows.
pub(crate) struct LightFades {
  pub whole: f32,
  pub faces: [f32; 6],
  pub shown: f32,
}

impl LightFades {
  /// Not faded at all, as an unshadowed light is.
  pub(crate) const WHOLE: Self = Self {
    whole: 1.0,
    faces: [1.0; 6],
    shown: 1.0,
  };

  /// How far a shadowed light has faded from `eye` over the cull's levels of detail, `start` to `end` metres.
  pub(crate) fn of(light: &LightDescription, basis: &LightBasis, eye: Vec3, (start, end): (f32, f32)) -> Self {
    if matches!(light.kind, LightKind::Spot) {
      let fade: f32 = LightBasis::get_light_lod(basis.get_spatial_sphere(light), eye, start, end);

      return Self {
        whole: fade,
        faces: [1.0; 6],
        shown: fade,
      };
    }

    let faces: [f32; 6] = LIGHT_SHADOW_POINT_FACES
      .map(|(direction, _)| LightBasis::get_light_lod(basis.get_face_sphere(light, direction), eye, start, end));

    Self {
      whole: 1.0,
      faces,
      shown: faces.into_iter().fold(0.0, f32::max),
    }
  }
}
