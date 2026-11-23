/// What a placed particle system is and plays, by the names `particles.xr` gives its effects and groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderParticleSource {
  /// One the level plants, `level.ps_static`, playing from load.
  Static { name: String },
  /// A zone's `idle_particles`, playing while the zone is enabled.
  Zone { idle: String },
  /// A campfire (`CZoneCampfire`): its idle effect while lit, `disabled_particles` while out, and
  /// `enabling_particles` once as it is lit.
  Campfire {
    /// Its spawned object's id, which its idle light names too.
    id: u16,
    idle: String,
    disabled: String,
    enabling: String,
  },
  /// An artefact's `particles` (`CArtefact::SwitchAfParticles`), playing while it shows: always, or for one a detector
  /// must reveal, only while the view shows those.
  Artefact { idle: String, is_detector_hidden: bool },
}
