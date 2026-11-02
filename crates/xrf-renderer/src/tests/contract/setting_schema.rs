use std::collections::BTreeMap;

use crate::contract::feature::lighting::render_shadow_settings::{RENDER_MAX_SHADOW_CASCADES, RenderShadowSettings};
use crate::contract::feature::weather::fog::render_fog_mode::RenderFogMode;
use crate::contract::settings::schema::render_setting_field::{RenderSettingField, RenderSettingNumber};
use crate::contract::settings::schema::render_setting_node::RenderSettingNode;
use crate::contract::settings::schema::render_setting_schema::RenderSettingSchema;
use crate::contract::view::render_view_features::RenderViewFeatures;

fn as_group(node: RenderSettingNode) -> BTreeMap<String, RenderSettingNode> {
  match node {
    RenderSettingNode::Group(group) => group,
    RenderSettingNode::Field(field) => panic!("a group, not {field:?}"),
  }
}

#[test]
fn describes_each_field_under_its_serialized_name_by_its_bounds_or_its_type() {
  let shadows: BTreeMap<String, RenderSettingNode> = as_group(RenderShadowSettings::describe());

  assert_eq!(shadows["isEnabled"], RenderSettingNode::Field(RenderSettingField::Flag));
  assert_eq!(
    shadows["resolution"],
    RenderSettingNode::Field(RenderSettingField::Number(RenderSettingNumber {
      min: 256.0,
      max: 8192.0,
      is_integer: true,
      recommended: Some([1024.0, 4096.0]),
      wasted_above: None,
    }))
  );
  assert_eq!(
    shadows["cascades"],
    RenderSettingNode::Field(RenderSettingField::Widths {
      min: 1.0,
      max: 2000.0,
      most: RENDER_MAX_SHADOW_CASCADES as u32,
    })
  );
  assert!(matches!(shadows["contact"], RenderSettingNode::Group(_)));
}

#[test]
fn offers_a_choice_of_each_variant_as_it_serializes() {
  assert_eq!(
    RenderFogMode::describe(),
    RenderSettingNode::Field(RenderSettingField::Choice {
      values: vec![String::from("engine"), String::from("enhanced")],
    })
  );
}

#[test]
fn leaves_out_what_the_settings_do_not_set() {
  let features: BTreeMap<String, RenderSettingNode> = as_group(RenderViewFeatures::describe());

  for skipped in ["bloom", "sunShafts", "corrections", "lightScales", "hemiStrength"] {
    assert!(!features.contains_key(skipped), "{skipped} is not a setting");
  }

  assert!(features.contains_key("sunTint"));
}
