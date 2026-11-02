use std::collections::BTreeMap;

use crate::contract::settings::render_settings::RenderSettings;
use crate::contract::settings::schema::render_setting_node::RenderSettingNode;
use crate::contract::settings::schema::render_setting_schema::RenderSettingSchema;
use crate::contract::view::output::render_view_output::RenderViewOutput;
use crate::contract::view::render_view_features::RenderViewFeatures;

/// Every setting a tool chooses its viewports' features by, what it takes and between which bounds: the view's
/// features the settings set, the output's upscaling, and the textures' filtering and quality every viewport shares.
#[xrf_ipc::constant]
pub fn render_feature_schema() -> RenderSettingNode {
  let mut group: BTreeMap<String, RenderSettingNode> = BTreeMap::new();

  for part in [
    RenderViewFeatures::describe(),
    RenderViewOutput::describe(),
    RenderSettings::describe(),
  ] {
    let RenderSettingNode::Group(settings) = part else {
      unreachable!("a struct describes a group");
    };

    group.extend(settings);
  }

  RenderSettingNode::Group(group)
}

/// Each feature as a view draws it until the settings set it: the engine's own, with the enhanced features' strengths
/// for when each is drawn.
#[xrf_ipc::constant]
pub fn render_default_view_features() -> RenderViewFeatures {
  RenderViewFeatures::default()
}

/// What a view's frame is resolved at and upscaled by until the settings set it.
#[xrf_ipc::constant]
pub fn render_default_view_output() -> RenderViewOutput {
  RenderViewOutput::default()
}

/// What every viewport draws with until the settings set it.
#[xrf_ipc::constant]
pub fn render_default_settings() -> RenderSettings {
  RenderSettings::default()
}
