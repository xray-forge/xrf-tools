use serde::Serialize;

use crate::diagnostics::memory::{GpuMemoryUsage, ProcessMemory, WebviewProcess, WebviewProcessMemory};

/// What the application and its webview hold in memory at one instant, and what the application holds on the GPU.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryUsage {
  /// The backend process itself.
  pub application: ProcessMemory,
  /// Every webview process still running, in the order the environment listed them.
  pub webview: Vec<WebviewProcessMemory>,
  /// The application's GPU memory, none while no renderer runs a GPU.
  pub gpu: Option<GpuMemoryUsage>,
}

impl MemoryUsage {
  /// Read the backend and each listed webview process beside the GPU's reading, or nothing where this platform cannot
  /// read a process.
  pub fn read(pid: u32, processes: &[WebviewProcess], gpu: Option<GpuMemoryUsage>) -> Option<Self> {
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
      gpu,
    })
  }
}
