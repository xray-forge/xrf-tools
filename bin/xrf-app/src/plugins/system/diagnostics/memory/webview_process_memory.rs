use serde::Serialize;

use crate::plugins::system::diagnostics::memory::{ProcessMemory, WebviewProcessKind};

/// What one webview process holds.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebviewProcessMemory {
  pub kind: WebviewProcessKind,
  /// The process's own identifier, for pairing it with a task manager.
  pub pid: u32,
  pub memory: ProcessMemory,
}
