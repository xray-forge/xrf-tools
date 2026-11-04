use std::time::Duration;

use xrf_renderer::{RenderHelperObject, RenderHelperShape};
use xrf_world::{WorldSound, WorldSoundInfo, WorldStaticSound, WorldZoneSound};

use crate::objects::{LEVEL_OBJECT_KINDS, LEVEL_SOUND_EMITTERS_HELPER_KIND};
use crate::sound::LevelSoundEmitters;

fn found(reference: &str, min_distance: f32, max_distance: f32) -> WorldSound {
  WorldSound::new(
    reference,
    Some(WorldSoundInfo {
      length: Duration::from_secs(2),
      min_distance,
      max_distance,
      ..WorldSoundInfo::default()
    }),
  )
}

fn zone(object: Option<u32>, idle: Option<WorldSound>, disabled: Option<WorldSound>) -> WorldZoneSound {
  WorldZoneSound {
    id: 1,
    object,
    position: [10.0, 1.0, 5.0],
    idle,
    is_campfire: disabled.is_some(),
    disabled,
    motion: None,
    zone_sphere: None,
  }
}

fn spheres(helper: &RenderHelperObject) -> Vec<f32> {
  helper
    .shapes
    .iter()
    .map(|shape| match shape {
      RenderHelperShape::Sphere { center, radius } if *center == [0.0; 3] => *radius,
      shape => panic!("a sphere about the emitter, not {shape:?}"),
    })
    .collect()
}

// Each sound's spheres stand where it plays, in renderer space, named as its object; a sound whose file is missing or
// whose object is not listed draws none, and a campfire draws its idle and its disabled sound alike.
#[test]
fn draws_each_emitters_spheres_at_its_place_named_as_its_object() {
  let emitters: LevelSoundEmitters = LevelSoundEmitters {
    statics: vec![
      WorldStaticSound {
        sound: found("ambient\\water", 2.0, 30.0),
        object: Some(9),
        position: [1.0, 2.0, 3.0],
        volume: 1.0,
        frequency: 1.0,
        active: [0, 0],
        play: [0, 0],
        pause: [0, 0],
      },
      WorldStaticSound {
        sound: WorldSound::new("ambient\\gone", None),
        object: Some(10),
        position: [0.0; 3],
        volume: 1.0,
        frequency: 1.0,
        active: [0, 0],
        play: [0, 0],
        pause: [0, 0],
      },
    ],
    zones: vec![
      zone(Some(4), Some(found("anomaly\\hum", 1.0, 20.0)), None),
      zone(None, Some(found("anomaly\\hum", 1.0, 20.0)), None),
      zone(
        Some(5),
        Some(found("ambient\\fire", 0.5, 15.0)),
        Some(found("ambient\\fire_out", 0.5, 10.0)),
      ),
    ],
  };
  let helpers: Vec<RenderHelperObject> = emitters.to_helpers();

  assert_eq!(
    helpers.iter().map(|helper| helper.id).collect::<Vec<u32>>(),
    [9, 4, 5, 5]
  );
  assert!(
    helpers
      .iter()
      .all(|helper| helper.kind == LEVEL_SOUND_EMITTERS_HELPER_KIND)
  );
  assert_eq!(LEVEL_SOUND_EMITTERS_HELPER_KIND as usize, LEVEL_OBJECT_KINDS.len());
  assert_eq!(spheres(&helpers[0]), [2.0, 30.0]);
  assert_eq!(spheres(&helpers[3]), [0.5, 10.0]);
  assert_eq!(helpers[0].transform[12..15], [1.0, 2.0, -3.0]);
  assert_eq!(helpers[1].transform[12..15], [10.0, 1.0, -5.0]);
}
