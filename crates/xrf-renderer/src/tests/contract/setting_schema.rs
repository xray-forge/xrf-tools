use std::collections::BTreeMap;

use crate::contract::feature::lighting::render_shadow_settings::{RENDER_MAX_SHADOW_CASCADES, RenderShadowSettings};
use crate::contract::feature::weather::fog::render_fog_mode::RenderFogMode;
use crate::contract::settings::render_settings::RenderSettings;
use crate::contract::settings::schema::render_setting_field::{RenderSettingField, RenderSettingNumber};
use crate::contract::settings::schema::render_setting_node::RenderSettingNode;
use crate::contract::settings::schema::render_setting_schema::RenderSettingSchema;
use crate::contract::view::render_view_features::RenderViewFeatures;
use crate::contract::view::render_view_options::RenderViewOptions;

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
fn holds_each_setting_to_its_bounds_and_leaves_what_the_settings_do_not_set() {
  let mut held: RenderViewFeatures = RenderViewFeatures::default();
  let hemi_strength: f32 = 1e6;

  held.shadows.cascades = vec![0.0, f32::NAN, 5000.0, 30.0, 60.0];
  held.shadows.resolution = 1 << 20;
  held.shadows.filter = 9;
  held.shadows.bias = -1.0;
  held.shadows.sway_rate = 22.6;
  held.hemi_strength = hemi_strength;

  held.hold_to_bounds();

  assert_eq!(held.shadows.cascades, [1.0, 1.0, 2000.0, 30.0]);
  assert_eq!(held.shadows.resolution, 8192);
  assert_eq!(held.shadows.filter, 3);
  assert_eq!(held.shadows.bias, 0.0);
  assert_eq!(held.shadows.sway_rate, 23.0);
  assert_eq!(held.hemi_strength, hemi_strength, "the world sets it, not the settings");
}

#[test]
fn holds_every_default_where_it_stands() {
  let mut options: RenderViewOptions = RenderViewOptions::default();
  let mut settings: RenderSettings = RenderSettings::default();

  options.hold_to_bounds();
  settings.hold_to_bounds();

  assert_eq!(options, RenderViewOptions::default());
  assert_eq!(settings, RenderSettings::default());
}

#[test]
fn leaves_out_what_the_settings_do_not_set() {
  let features: BTreeMap<String, RenderSettingNode> = as_group(RenderViewFeatures::describe());

  for skipped in ["bloom", "sunShafts", "corrections", "lightScales", "hemiStrength"] {
    assert!(!features.contains_key(skipped), "{skipped} is not a setting");
  }

  assert!(features.contains_key("sunTint"));
}
