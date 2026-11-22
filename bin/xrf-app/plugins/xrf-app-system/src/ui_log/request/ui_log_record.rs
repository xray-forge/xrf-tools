use serde::{Deserialize, Serialize};
use xrf_app_core::logging::UiLogEntry;

use crate::ui_log::UiLogLevel;

/// One record the webview logged, as it sends it in a batch.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UiLogRecord {
  pub level: UiLogLevel,
  /// The logger's scope, written under the target `xrf_ui::<scope>`; an empty one writes under `xrf_ui` itself.
  pub scope: String,
  /// Milliseconds since the Unix epoch at which the webview logged it, which its line is stamped with.
  pub time: u64,
  /// The text, a stack's many lines included.
  pub message: String,
}

impl UiLogRecord {
  /// The record as the UI log writes it.
  pub(crate) fn as_entry(&self) -> UiLogEntry<'_> {
    UiLogEntry {
      level: self.level.into(),
      scope: &self.scope,
      time: self.time,
      message: &self.message,
    }
  }
}
