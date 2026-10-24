// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `ipc` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::ipc` describes the sections.
macro_rules! for_each_visuals_domain {
  ($consumer:ident) => {
    $consumer! {
      visuals => "visuals" {
        close_browse => crate::commands::close_browse::visuals_close_browse,
        close_model => crate::commands::close_model::visuals_close_model,
        close_motion => crate::commands::close_motion::visuals_close_motion,
        describe_motion => crate::commands::describe_motion::visuals_describe_motion,
        list_motions => crate::commands::list_motions::visuals_list_motions,
        open_browse => crate::commands::open_browse::visuals_open_browse,
        open_model => crate::commands::open_model::visuals_open_model,
        open_motion => crate::commands::open_motion::visuals_open_motion,
        restore_browse => crate::commands::restore_browse::visuals_restore_browse,
        restore_model => crate::commands::restore_model::visuals_restore_model,
        show_model => crate::commands::show_model::visuals_show_model,
      }
    }
  };
}
