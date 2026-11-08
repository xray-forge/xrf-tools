use std::collections::BTreeMap;

use serde::Serialize;

use crate::contract::settings::schema::render_setting_field::RenderSettingField;

/// One node of the settings' description: a setting's field, or a group of nodes by each one's serialized name.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub(crate) enum RenderSettingNode {
  Field(RenderSettingField),
  #[expect(
    clippy::use_self,
    reason = "Specta's derive copies the type where `Self` names another"
  )]
  Group(BTreeMap<String, RenderSettingNode>),
}
