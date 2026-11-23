//! Holds which spawn groups the toggles leave out of a scene, and that the objects of no group always stand.

use std::collections::HashSet;

use xrf_renderer::{RenderSpawnCategory, RenderSpawnGroup};

use crate::contract::world_toggles::WorldToggles;
use crate::level::streaming::hidden_spawn_groups::HiddenSpawnGroups;

/// The group of `category`'s objects a new game keeps and nothing hides until detected.
fn kept(category: RenderSpawnCategory) -> RenderSpawnGroup {
  RenderSpawnGroup {
    category,
    is_released: false,
    is_detector_hidden: false,
  }
}

fn released(category: RenderSpawnCategory) -> RenderSpawnGroup {
  RenderSpawnGroup {
    is_released: true,
    ..kept(category)
  }
}

fn detector_hidden(group: RenderSpawnGroup) -> RenderSpawnGroup {
  RenderSpawnGroup {
    is_detector_hidden: true,
    ..group
  }
}

#[test]
fn stands_the_objects_of_no_group_whatever_is_hidden() {
  let hidden: HiddenSpawnGroups = (0..=16).fold(HiddenSpawnGroups::default(), HiddenSpawnGroups::with);

  assert!(hidden.is_shown(0));
  assert!(!hidden.is_shown(1));
  assert!(!hidden.is_shown(16));
}

#[test]
fn numbers_every_group_apart_from_one_to_sixteen() {
  let indices: HashSet<u32> = RenderSpawnGroup::list_all().map(RenderSpawnGroup::get_index).collect();

  assert_eq!(indices, (1..=16).collect());
}

#[test]
fn hides_only_the_released_objects_by_default() {
  let hidden: HiddenSpawnGroups = WorldToggles::default().get_hidden_spawn_groups();

  for category in RenderSpawnCategory::ALL {
    assert!(hidden.is_shown(kept(category).get_index()), "{category:?} kept");
    assert!(
      hidden.is_shown(detector_hidden(kept(category)).get_index()),
      "{category:?} detector-hidden"
    );
    assert!(
      !hidden.is_shown(released(category).get_index()),
      "{category:?} released"
    );
    assert!(
      !hidden.is_shown(detector_hidden(released(category)).get_index()),
      "{category:?} released detector-hidden"
    );
  }
}

#[test]
fn leaves_out_an_object_hidden_one_by_one_whatever_its_group_shows() {
  let toggles: WorldToggles = WorldToggles {
    hidden_spawn_objects: vec![3, 9],
    ..WorldToggles::default()
  };
  let filter = toggles.get_spawn_filter();
  let props: u32 = kept(RenderSpawnCategory::Props).get_index();

  assert!(!filter.is_shown(props, 9));
  assert!(filter.is_shown(props, 4));
  assert!(!filter.is_shown(released(RenderSpawnCategory::Props).get_index(), 4));
}

#[test]
fn hides_a_category_switched_off_with_its_released_objects() {
  let toggles: WorldToggles = WorldToggles {
    is_spawned_weapons: false,
    is_spawned_released: true,
    ..WorldToggles::default()
  };
  let hidden: HiddenSpawnGroups = toggles.get_hidden_spawn_groups();

  assert!(!hidden.is_shown(kept(RenderSpawnCategory::Weapons).get_index()));
  assert!(!hidden.is_shown(released(RenderSpawnCategory::Weapons).get_index()));
  assert!(hidden.is_shown(kept(RenderSpawnCategory::Lamps).get_index()));
  assert!(hidden.is_shown(released(RenderSpawnCategory::Lamps).get_index()));
}

// The game at rest: an artefact a detector must reveal stands nowhere, released or kept, while the rest of its
// category stands.
#[test]
fn hides_the_detector_hidden_artefacts_switched_off_alone() {
  let toggles: WorldToggles = WorldToggles {
    is_spawned_detector_hidden: false,
    is_spawned_released: true,
    ..WorldToggles::default()
  };
  let hidden: HiddenSpawnGroups = toggles.get_hidden_spawn_groups();
  let items: RenderSpawnCategory = RenderSpawnCategory::Items;

  assert!(hidden.is_shown(kept(items).get_index()));
  assert!(hidden.is_shown(released(items).get_index()));
  assert!(!hidden.is_shown(detector_hidden(kept(items)).get_index()));
  assert!(!hidden.is_shown(detector_hidden(released(items)).get_index()));
}
