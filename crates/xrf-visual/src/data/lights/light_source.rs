use crate::data::lights::light_motion::LightMotion;
use crate::data::lights::zone_sphere::ZoneSphere;

/// What gives a light, with what only that source has: the object or file it belongs to, which decides how it is
/// switched, moved and how large it glows.
#[derive(Clone, Debug, PartialEq)]
pub enum LightSource {
  /// A hanging lamp's main or ambient light (`CHangingLamp`).
  Lamp,
  /// One of the level file's own lights, which the engine draws only with `r2_allow_r1_lights`.
  Level,
  /// An anomalous zone's idle light (`CCustomZone::StartIdleLight`).
  Zone {
    /// The motion carrying a torrid zone, which moves its light each frame. None for a zone that stands.
    motion: Option<LightMotion>,
    /// Its zone's sphere in renderer space, offset from the light: a torrid zone's, whose light goes out while the
    /// camera stands far from it (`o_switch_2_slow`, `CTorridZone::light_in_slow_mode`). None for one always on.
    sphere: Option<ZoneSphere>,
  },
  /// A campfire's idle light (`CZoneCampfire`), which its switching fades, by the spawned object's id.
  Campfire { id: u16 },
  /// An artefact's trail light (`CArtefact::StartLights`).
  Artefact {
    /// Whether its artefact lies hidden until a detector reveals it, and its light with it: lit only while a view shows
    /// those artefacts.
    is_detector_hidden: bool,
  },
}

impl LightSource {
  /// The spawned campfire whose switching fades the light, None for any other source.
  pub fn get_campfire(&self) -> Option<u16> {
    match self {
      Self::Campfire { id } => Some(*id),
      _ => None,
    }
  }

  /// The motion carrying a torrid zone's light, None for any light that stands.
  pub fn get_motion(&self) -> Option<&LightMotion> {
    match self {
      Self::Zone { motion, .. } => motion.as_ref(),
      _ => None,
    }
  }

  /// The sphere a torrid zone's light goes out by, None for any light always on.
  pub fn get_zone_sphere(&self) -> Option<&ZoneSphere> {
    match self {
      Self::Zone { sphere, .. } => sphere.as_ref(),
      _ => None,
    }
  }

  /// Whether the light is a detector-hidden artefact's.
  pub fn is_detector_hidden(&self) -> bool {
    matches!(
      self,
      Self::Artefact {
        is_detector_hidden: true
      }
    )
  }
}
