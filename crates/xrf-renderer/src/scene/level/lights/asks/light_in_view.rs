use glam::Vec4;
use xrf_renderer_core::ProxyHandle;
use xrf_visual::LightDescription;

use crate::lighting::lamp::light_basis::LightBasis;

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
}
