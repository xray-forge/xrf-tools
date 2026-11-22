use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::logging::UiLog;
use xrf_build_info::BuildInfo;

/// What the application makes before Tauri runs and every plugin's setup builds on, managed before any plugin.
pub(crate) struct ApplicationFoundation {
  pub build: BuildInfo,
  /// The process's one pool, which the renderer's loaders run on too.
  pub execution: ExecutionState,
  pub directories: AppDirectories,
  /// The run's UI log, opened with the logger beside the run's log file; closed where either file did not open.
  pub ui_log: UiLog,
}
