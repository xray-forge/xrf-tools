/// What gives a light: the object or file it belongs to, which decides how it is switched and how large it glows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightSource {
  /// A hanging lamp's main or ambient light (`CHangingLamp`).
  Lamp,
  /// One of the level file's own lights, which the engine draws only with `r2_allow_r1_lights`.
  Level,
  /// An anomalous zone's idle light (`CCustomZone::StartIdleLight`), a torrid zone's among them.
  Zone,
  /// A campfire's idle light (`CZoneCampfire`), which its switching fades, by the spawned object's id.
  Campfire { id: u16 },
  /// An artefact's trail light (`CArtefact::StartLights`).
  Artefact,
}

impl LightSource {
  /// The spawned campfire whose switching fades the light, None for any other source.
  pub fn get_campfire(&self) -> Option<u16> {
    match self {
      Self::Campfire { id } => Some(*id),
      _ => None,
    }
  }
}
