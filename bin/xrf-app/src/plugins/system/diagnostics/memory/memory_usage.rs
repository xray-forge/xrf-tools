use serde::Serialize;

use crate::plugins::system::diagnostics::memory::{ProcessMemory, WebviewProcess, WebviewProcessMemory};

/// What the application and its webview hold in memory at one instant.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryUsage {
  /// The backend process itself.
  pub application: ProcessMemory,
  /// Every webview process still running, in the order the environment listed them.
  pub webview: Vec<WebviewProcessMemory>,
}

impl MemoryUsage {
  /// Read the backend and each listed webview process, or nothing where this platform cannot read a process.
  pub fn read(pid: u32, processes: &[WebviewProcess]) -> Option<Self> {
    Some(Self {
      application: ProcessMemory::read(pid)?,
      // A process listed a moment ago may have exited since; it holds nothing now, so it is left out.
      webview: processes
        .iter()
        .filter_map(|process| {
          ProcessMemory::read(process.pid).map(|memory| WebviewProcessMemory {
            kind: process.kind,
            pid: process.pid,
            memory,
          })
        })
        .collect(),
    })
  }
}
