use serde::Serialize;

use crate::core::types::TauriResult;
use crate::core::window::{WebviewOptions, WebviewOptionsState};

/// The webview options this run started with, and the ones the next start applies.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebviewOptionsStatus {
  pub running: WebviewOptions,
  pub chosen: WebviewOptions,
}

impl WebviewOptionsStatus {
  /// Read both off the state the main window was built with.
  pub fn read(state: &WebviewOptionsState) -> TauriResult<Self> {
    Ok(Self {
      running: state.get_running(),
      chosen: state.get_chosen()?,
    })
  }
}
