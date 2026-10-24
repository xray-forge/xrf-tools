// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `ipc` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::ipc` describes the sections.
macro_rules! for_each_exports_domain {
  ($consumer:ident) => {
    $consumer! {
      exports => "exports" {
        close_project => crate::commands::close_project::exports_close_project,
        export_manifest => crate::commands::export_manifest::exports_export_manifest,
        get_source => crate::commands::get_source::exports_get_source,
        open_project => crate::commands::open_project::exports_open_project,
        restore_project => crate::commands::restore_project::exports_restore_project,
      }
    }
  };
}
