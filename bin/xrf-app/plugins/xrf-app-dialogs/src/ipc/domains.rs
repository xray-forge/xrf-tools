// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `ipc` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::ipc` describes the sections.
macro_rules! for_each_dialogs_domain {
  ($consumer:ident) => {
    $consumer! {
      dialogs => "dialogs" {
        close_project => crate::commands::close_project::dialogs_close_project,
        detect_mode => crate::commands::detect_mode::dialogs_detect_mode,
        get_dialog => crate::commands::get_dialog::dialogs_get_dialog,
        open_project => crate::commands::open_project::dialogs_open_project,
        restore_project => crate::commands::restore_project::dialogs_restore_project,
      }
    }
  };
}
