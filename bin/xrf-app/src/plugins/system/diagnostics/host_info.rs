//! The machine and the runtime the application found when it started.

use std::thread;

use serde::Serialize;
use sysinfo::{MemoryRefreshKind, System};

/// What the application is running on and with, none of which changes while it runs.
///
/// Split from [`RuntimeSnapshot`](super::RuntimeSnapshot) because that one is polled: re-reading the operating
/// system's name every second to show the same string is work nobody asked for, and mixing a constant into a reading
/// invites a surface to refresh the wrong half.
///
/// Every field an operating system may decline to report is `Option`, the way `BuildInfo` treats what a build could
/// not record - naming the absence beats substituting a plausible default.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInfo {
  /// Tauri the application was linked against.
  pub tauri_version: &'static str,
  /// Webview actually serving the window, which is the runtime installed on the machine rather than a compiled-in
  /// version. Absent where the platform cannot be asked, and on Windows where no WebView2 runtime answered.
  pub webview_version: Option<String>,
  /// Operating system's short name, such as `Windows` or `Ubuntu`.
  pub os_name: Option<String>,
  /// Operating system's own version, as it numbers itself.
  pub os_version: Option<String>,
  /// Kernel behind it, which on Windows is the build number a compatibility report is quoted by.
  pub kernel_version: Option<String>,
  /// Architecture the binary is executing on, as opposed to the target triple it was built for.
  pub arch: String,
  /// Logical processors, which is what the execution pool's width is drawn from.
  pub cpu_count: u32,
  /// Physical cores, absent where the platform does not distinguish them.
  pub physical_core_count: Option<u32>,
  /// Total physical memory of the machine, the figure every usage reading is read against.
  pub total_memory: u64,
  /// This process's own identifier, for pairing what is shown here with a task manager.
  pub pid: u32,
}

impl HostInfo {
  /// Reads what the application is running on and with.
  ///
  /// Builds its own reader rather than borrowing the retained one: this is answered once per window, and the reader
  /// the usage snapshot lends is refreshed for processes, which this needs none of.
  pub fn read() -> Self {
    Self {
      tauri_version: tauri::VERSION,
      // The one reading that comes from the webview rather than from the operating system, and the one most worth
      // having in a bug report: a rendering fault on this stack is usually the installed runtime's, not ours.
      webview_version: tauri::webview_version().ok(),
      os_name: System::name(),
      os_version: System::os_version(),
      kernel_version: System::kernel_version(),
      arch: System::cpu_arch(),
      cpu_count: logical_cpu_count(),
      physical_core_count: System::physical_core_count().map(|it| it as u32),
      total_memory: total_memory(),
      pid: std::process::id(),
    }
  }
}

/// Logical processors visible to this process, which is the figure the execution pool sizes itself from.
fn logical_cpu_count() -> u32 {
  thread::available_parallelism().map_or(0, |it| it.get() as u32)
}

/// Physical memory installed, read through a throwaway view because it is the only machine-wide figure wanted here.
fn total_memory() -> u64 {
  let mut system: System = System::new();

  system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());

  system.total_memory()
}
