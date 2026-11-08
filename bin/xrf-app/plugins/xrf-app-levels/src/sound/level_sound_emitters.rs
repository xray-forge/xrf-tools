use std::collections::HashMap;
use std::sync::Arc;

use glam::{Mat4, Vec3};
use xrf_app_core::session::SessionSnapshot;
use xrf_level::{LevelSndStaticFile, LevelSndStaticSound, LevelSndStaticWindow};
use xrf_ltx::{Ltx, Section};
use xrf_renderer::{RenderHelperObject, RenderHelperShape};
use xrf_spawn::{AlifeObject, AlifeObjectCustomZone, AlifeObjectInherited, AlifeObjectMotion, ClsId};
use xrf_vfs::XraySearch;
use xrf_visual::ZoneSphere;
use xrf_world::{WorldSound, WorldStaticSound, WorldZoneSound};

use crate::level::SelectedLevel;
use crate::objects::{LEVEL_SOUND_EMITTERS_HELPER_KIND, LevelObjectId, LevelObjectsDescription, LevelPlacementFile};
use crate::parts::{LevelPlacements, LevelPlacementsPart, LevelSectionsPart, LevelSpawn, LevelSpawnPart};
use crate::report::LevelReport;
use crate::sound::LevelSoundFiles;

/// What a level plays at points of its own whatever its weather: its static sounds (`level.snd_static`) and its zones'
/// sounds (`CCustomZone`'s `idle_sound`, `CZoneCampfire`'s `disabled_sound`), each named by its level object's place in
/// the level's listing, which its helper carries.
pub(crate) struct LevelSoundEmitters {
  pub statics: Vec<WorldStaticSound>,
  pub zones: Vec<WorldZoneSound>,
}

impl LevelSoundEmitters {
  /// The sound emitters' helpers' colour, sRGB from 0 to 1: amber, apart from every kind's.
  const HELPER_COLOR: [f32; 3] = [1.0, 0.7, 0.0];

  /// Metres from the camera past which an emitter's spheres fade out.
  const HELPER_DRAW_DISTANCE: f32 = 300.0;

  /// Reads the level's sound emitters through `files`, each named by its place among `objects`, none where the level's
  /// objects could not be listed; a level whose spawn or configs cannot be read sounds no zone.
  pub(crate) fn read(
    level: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    objects: Option<&LevelObjectsDescription>,
    files: &mut LevelSoundFiles<'_>,
  ) -> Self {
    let places: HashMap<LevelObjectId, u32> = objects
      .into_iter()
      .flat_map(|objects| objects.objects.iter().enumerate())
      .map(|(place, object)| (object.id, place as u32))
      .collect();
    let placements: Arc<LevelPlacements> = LevelPlacementsPart::get(level, search);
    let statics: Vec<WorldStaticSound> = placements
      .sounds
      .iter()
      .flat_map(|file: &LevelSndStaticFile| file.sounds.iter().enumerate())
      .map(|(index, sound)| {
        let id: LevelObjectId = LevelObjectId::Placement {
          file: LevelPlacementFile::Sounds,
          index: index as u32,
        };

        Self::read_static(sound, places.get(&id).copied(), files)
      })
      .collect();
    let zones: Vec<WorldZoneSound> = match (
      LevelSpawnPart::get(level, search),
      LevelSectionsPart::get(level, search),
    ) {
      (Ok(spawn), Ok(sections)) => Self::read_zones(&spawn, &sections, &places, files),
      (Err(error), _) | (_, Err(error)) => {
        LevelReport::missing_zone_sounds(&level.source, &error);

        Vec::new()
      }
    };

    Self { statics, zones }
  }

  /// Each emitter's spheres out to its sound's minimum and maximum distance where it stands, as the helpers the sound
  /// emitters' kind switches, named as its level object is; none for a sound whose file is missing.
  pub(crate) fn to_helpers(&self) -> Vec<RenderHelperObject> {
    let statics = self
      .statics
      .iter()
      .map(|sound| (sound.object, sound.position, &sound.sound));
    let zones = self.zones.iter().flat_map(|zone| {
      [&zone.idle, &zone.disabled]
        .into_iter()
        .flatten()
        .map(|sound| (zone.object, zone.position, sound))
    });
    let mut helpers: Vec<RenderHelperObject> = Vec::new();

    for (object, [x, y, z], sound) in statics.chain(zones) {
      let (Some(object), Some(info)) = (object, sound.info) else {
        continue;
      };
      helpers.push(RenderHelperObject {
        id: object,
        kind: LEVEL_SOUND_EMITTERS_HELPER_KIND,
        color: Self::HELPER_COLOR,
        draw_distance: Self::HELPER_DRAW_DISTANCE,
        // In renderer space, as every helper stands.
        transform: Mat4::from_translation(Vec3::new(x, y, -z)).to_cols_array(),
        shapes: [info.min_distance, info.max_distance]
          .into_iter()
          .filter(|radius| *radius > 0.0)
          .map(|radius| RenderHelperShape::Sphere {
            center: [0.0; 3],
            radius,
          })
          .collect(),
      });
    }

    helpers
  }

  fn read_static(
    sound: &LevelSndStaticSound,
    object: Option<u32>,
    files: &mut LevelSoundFiles<'_>,
  ) -> WorldStaticSound {
    let window = |window: &LevelSndStaticWindow| [window.from, window.to];

    WorldStaticSound {
      sound: files.get(&sound.sound),
      object,
      position: [sound.position.x, sound.position.y, sound.position.z],
      volume: sound.volume,
      frequency: sound.frequency,
      active: window(&sound.active),
      play: window(&sound.play),
      pause: window(&sound.pause),
    }
  }

  /// `CCustomZone::Load`: every zone a new game keeps whose section names an `idle_sound`, and every campfire, in the
  /// spawn's order.
  fn read_zones(
    spawn: &LevelSpawn,
    sections: &Ltx,
    places: &HashMap<LevelObjectId, u32>,
    files: &mut LevelSoundFiles<'_>,
  ) -> Vec<WorldZoneSound> {
    spawn
      .objects
      .iter()
      .enumerate()
      .filter(|(index, _)| spawn.get_release(*index).is_none())
      .filter_map(|(index, object)| {
        let id: LevelObjectId = LevelObjectId::Spawn { index: index as u32 };

        Self::find_zone(object, sections, places.get(&id).copied(), files)
      })
      .collect()
  }

  fn find_zone(
    object: &AlifeObject,
    sections: &Ltx,
    place: Option<u32>,
    files: &mut LevelSoundFiles<'_>,
  ) -> Option<WorldZoneSound> {
    let (motion, zone): (Option<&AlifeObjectMotion>, &AlifeObjectCustomZone) = match &object.inherited {
      AlifeObjectInherited::CseAlifeAnomalousZone(zone) => (None, &zone.base.base),
      AlifeObjectInherited::CseAlifeZoneVisual(zone) => (None, &zone.base.base),
      AlifeObjectInherited::CseAlifeTorridZone(zone) => (Some(&zone.motion), &zone.base),
      _ => return None,
    };
    let section: &Section = sections.section(&object.section)?;
    let read = |key: &str| section.get(key).map(str::trim).filter(|value| !value.is_empty());
    let is_campfire: bool = read("class").and_then(ClsId::from_class_name) == Some(ClsId::ZCFire);
    let idle: Option<WorldSound> = read("idle_sound").map(|sound| files.get(sound));
    let disabled: Option<WorldSound> = read("disabled_sound")
      .filter(|_| is_campfire)
      .map(|sound| files.get(sound));

    if idle.is_none() && disabled.is_none() {
      return None;
    }

    Some(WorldZoneSound {
      id: object.id,
      object: place,
      position: [object.position.x, object.position.y, object.position.z],
      idle,
      is_campfire,
      disabled,
      motion: motion.and_then(AlifeObjectMotion::get_name).map(str::to_owned),
      zone_sphere: ZoneSphere::of_custom_zone(zone, (&object.position, &object.direction)),
    })
  }
}
