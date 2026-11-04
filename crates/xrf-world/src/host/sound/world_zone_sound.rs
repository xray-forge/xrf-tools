use xrf_visual::ZoneSphere;

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::world_sound::WorldSound;

/// One of a level's zones that sounds (`CCustomZone`): its section's `idle_sound`, a campfire's `disabled_sound`, where
/// it stands in engine space, the motion that carries it, and the sphere its distance from the camera is measured to.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldZoneSound {
  /// The spawned object's id, which a campfire is switched by.
  pub id: u16,
  /// The level object it is, by the id its helper carries; none where the level's objects could not be listed.
  pub object: Option<u32>,
  pub position: [f32; 3],
  /// Looped while its idle plays; none for a zone whose section names none.
  pub idle: Option<WorldSound>,
  /// Whether it is a campfire (`CZoneCampfire`), whose idle follows its switching.
  pub is_campfire: bool,
  /// A campfire's `disabled_sound`, looped while it is out.
  pub disabled: Option<WorldSound>,
  /// The object motion a torrid zone follows (`motion_name`).
  pub motion: Option<String>,
  /// None for a zone always fast (`[fast_mode] always_fast`), which never slows.
  pub zone_sphere: Option<ZoneSphere>,
}

impl WorldZoneSound {
  /// What plays its sounds: a campfire, or any other zone.
  pub fn get_kind(&self) -> WorldSoundKind {
    if self.is_campfire {
      WorldSoundKind::Campfire
    } else {
      WorldSoundKind::Anomaly
    }
  }
}
