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
    idle: String,
    disabled: String,
    enabling: String,
  },
}
