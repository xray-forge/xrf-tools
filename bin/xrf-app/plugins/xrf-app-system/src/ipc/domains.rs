// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `ipc` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::ipc` describes the sections.
macro_rules! for_each_system_domain {
  ($consumer:ident) => {
    $consumer! {
      system => "system" {
        approve_window_close => crate::window::commands::approve_window_close::system_approve_window_close,
        clear_app_storage => crate::paths::commands::clear_app_storage::system_clear_app_storage,
        describe_path => crate::paths::commands::describe_path::system_describe_path,
        get_app_data_root => crate::paths::commands::get_app_data_root::system_get_app_data_root,
        get_app_storage => crate::paths::commands::get_app_storage::system_get_app_storage,
        get_build_info => crate::diagnostics::commands::get_build_info::system_get_build_info,
        get_default_output_root => crate::paths::commands::get_default_output_root::system_get_default_output_root,
        get_environment_flags => crate::diagnostics::commands::get_environment_flags::system_get_environment_flags,
        get_host_info => crate::diagnostics::commands::get_host_info::system_get_host_info,
        get_memory_usage => crate::diagnostics::commands::get_memory_usage::system_get_memory_usage,
        get_runtime_snapshot => crate::diagnostics::commands::get_runtime_snapshot::system_get_runtime_snapshot,
        report_leave_decision => crate::window::commands::report_leave_decision::system_report_leave_decision,
        reveal_path => crate::paths::commands::reveal_path::system_reveal_path,
        reveal_window => crate::window::commands::reveal_window::system_reveal_window,
        tidy_app_cache => crate::paths::commands::tidy_app_cache::system_tidy_app_cache,
      }
    }
  };
}
