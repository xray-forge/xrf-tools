//! What names a scene's tools and its objects' typed values as they cross to the page: each tool by the name its part
//! files carry, in the editor's order, and each value the editor saves as a number by what it stands for.

use xrf_editable_level::{
  EditableFogVolumeKind, EditableFogVolumeObject, EditableSectorObject, EditableShape, EditableWayObject,
};

use crate::scene::{
  SceneBinaryToolDescription, SceneLightType, SceneObjectProperties, SceneShapeUsage, SceneToolKind, SceneWayType,
};
use crate::tests::fixtures::scene_custom_object;

fn spell<T: serde::Serialize>(value: &T) -> serde_json::Value {
  serde_json::to_value(value).expect("the value serializes")
}

#[test]
fn each_tool_crosses_as_the_name_its_part_files_carry_in_the_editors_order() {
  for kind in SceneToolKind::ALL {
    assert_eq!(spell(&kind), kind.get_name(), "{kind:?}");
    assert_eq!(SceneToolKind::of(kind.get_name()), Some(kind));
  }

  assert!(
    SceneToolKind::ALL.is_sorted(),
    "the list of tools is in the order they are declared"
  );
  assert_eq!(
    SceneToolKind::ALL.map(SceneToolKind::get_name),
    [
      "group",
      "glow",
      "scene_object",
      "light",
      "shape",
      "sound_src",
      "spawn",
      "way",
      "sector",
      "portal",
      "sound_env",
      "ps",
      "detail_object",
      "ai_map",
      "wallmark",
      "fog_volume",
    ],
    "the editor's class ids, `ObjClassID`"
  );
  assert_eq!(SceneToolKind::of("custom_tool"), None);
}

#[test]
fn a_binary_tools_counts_name_their_tool_as_its_kind_does() {
  for (counted, kind) in [
    (SceneBinaryToolDescription::AiMap { nodes: 1 }, SceneToolKind::AiMap),
    (
      SceneBinaryToolDescription::DetailObject { objects: 1, slots: 2 },
      SceneToolKind::DetailObject,
    ),
    (
      SceneBinaryToolDescription::Wallmark { slots: 1, marks: 2 },
      SceneToolKind::Wallmark,
    ),
  ] {
    assert_eq!(spell(&counted)["tool"], spell(&kind), "{counted:?}");
  }
}

#[test]
fn a_saved_number_crosses_as_what_it_stands_for_and_one_the_editor_does_not_name_as_itself() {
  assert_eq!(SceneLightType::of(1), SceneLightType::Point);
  assert_eq!(SceneLightType::of(2), SceneLightType::Spot);
  assert_eq!(SceneLightType::of(3), SceneLightType::Directional);
  assert_eq!(SceneLightType::of(7), SceneLightType::Unknown { value: 7 });
  assert_eq!(
    SceneWayType::of(EditableWayObject::PATROL_PATH),
    SceneWayType::PatrolPath
  );
  assert_eq!(SceneWayType::of(4), SceneWayType::Unknown { value: 4 });
  assert_eq!(SceneShapeUsage::of(0), SceneShapeUsage::Common);
  assert_eq!(
    SceneShapeUsage::of(EditableShape::LEVEL_BOUND_USAGE),
    SceneShapeUsage::LevelBound
  );
  assert_eq!(SceneShapeUsage::of(9), SceneShapeUsage::Unknown { value: 9 });
  assert_eq!(
    spell(&SceneLightType::Unknown { value: 7 }),
    serde_json::json!({ "kind": "unknown", "value": 7 })
  );
}

#[test]
fn a_sector_switching_to_no_map_and_an_ungrouped_volume_cross_as_none() {
  let sector = |map_index: u8| EditableSectorObject {
    object: scene_custom_object("sector_0000", 0.0),
    color: [0, 0, 0, 255],
    is_default: false,
    items: Vec::new(),
    map_index,
  };
  let volume = |group: u32| EditableFogVolumeObject {
    object: scene_custom_object("fog_volume_0000", 0.0),
    shape: EditableShape::new_unit_box(),
    kind: EditableFogVolumeKind::Occlusion,
    group,
  };

  assert!(matches!(
    SceneObjectProperties::from(&sector(EditableSectorObject::NO_MAP)),
    SceneObjectProperties::Sector { map_index: None, .. }
  ));
  assert!(matches!(
    SceneObjectProperties::from(&sector(3)),
    SceneObjectProperties::Sector { map_index: Some(3), .. }
  ));
  assert!(matches!(
    SceneObjectProperties::from(&volume(EditableFogVolumeObject::UNGROUPED)),
    SceneObjectProperties::FogVolume { group: None, .. }
  ));
  assert!(matches!(
    SceneObjectProperties::from(&volume(0)),
    SceneObjectProperties::FogVolume { group: Some(0), .. }
  ));
}
