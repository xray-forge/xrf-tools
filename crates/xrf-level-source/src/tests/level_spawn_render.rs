//! Holds the groups the drawn spawn stands each object in: its category, its release, and whether its artefact's
//! section hides it until a detector reveals it.

use std::collections::HashMap;

use xrf_ltx::Ltx;
use xrf_renderer::{RenderLevelSpawn, RenderSpawnCategory, RenderSpawnGroup};
use xrf_spawn::{AlifeObject, AlifeObjectInherited, ClsId};

use crate::fixtures::{new_artefact, new_item_base, new_object};
use crate::parts::LevelSpawn;
use crate::spawn::LevelSpawnRelease;

/// A plain item whose section would hide it were it an artefact.
fn new_item(name: &str, section: &str) -> AlifeObject {
  new_object(
    name,
    section,
    ClsId::SFood,
    AlifeObjectInherited::CseAlifeItem(Box::new(new_item_base("dynamics\\artefacts\\af_medusa"))),
  )
}

fn sections() -> Ltx {
  Ltx::read_from_str(
    r"
[af_base]
can_be_controlled = true
af_rank = 1

[af_medusa]:af_base

[af_junk]:af_base
af_rank = 0
",
  )
  .and_then(Ltx::into_inherited)
  .expect("the sections to parse and inherit")
}

/// The spawn's objects: a ranked artefact, a junk one, an item of the ranked section, and a ranked artefact a new game
/// releases.
fn new_spawn() -> LevelSpawn {
  LevelSpawn {
    arrivals: Vec::new(),
    objects: vec![
      new_artefact("ranked", "af_medusa"),
      new_artefact("junk", "af_junk"),
      new_item("item", "af_medusa"),
      new_artefact("released", "af_medusa"),
    ],
    releases: HashMap::from([(3, LevelSpawnRelease::RemoveObjects)]),
  }
}

fn list_groups(spawn: &RenderLevelSpawn) -> Vec<RenderSpawnGroup> {
  spawn.objects.iter().map(|object| object.group).collect()
}

#[test]
fn marks_the_artefacts_their_sections_hide_until_detected_released_or_kept() {
  let ltx: Ltx = sections();
  let items: RenderSpawnGroup = RenderSpawnCategory::Items.into();
  let hidden: RenderSpawnGroup = RenderSpawnGroup {
    is_detector_hidden: true,
    ..items
  };

  assert_eq!(
    list_groups(&new_spawn().get_render(Some(&ltx))),
    [
      hidden,
      items,
      items,
      RenderSpawnGroup {
        is_released: true,
        ..hidden
      },
    ]
  );
}

#[test]
fn marks_no_artefact_without_the_configs_to_read_its_section() {
  let items: RenderSpawnGroup = RenderSpawnCategory::Items.into();

  assert_eq!(
    list_groups(&new_spawn().get_render(None)),
    [
      items,
      items,
      items,
      RenderSpawnGroup {
        is_released: true,
        ..items
      },
    ]
  );
}
