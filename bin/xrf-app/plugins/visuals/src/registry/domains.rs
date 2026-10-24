// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `registry` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::registry` describes the sections.
macro_rules! for_each_visuals_domain {
  ($consumer:ident) => {
    $consumer! {
      visuals => "visuals" {
        close_browse => crate::commands::close_browse::visuals_close_browse,
        close_model => crate::commands::close_model::visuals_close_model,
        close_motion => crate::commands::close_motion::visuals_close_motion,
        describe_motion => crate::commands::describe_motion::visuals_describe_motion,
        get_browse => crate::commands::get_browse::visuals_get_browse,
        get_model => crate::commands::get_model::visuals_get_model,
        list_motions => crate::commands::list_motions::visuals_list_motions,
        open_browse => crate::commands::open_browse::visuals_open_browse,
        open_model => crate::commands::open_model::visuals_open_model,
        open_motion => crate::commands::open_motion::visuals_open_motion,
        show_model => crate::commands::show_model::visuals_show_model,
      }
    }
  };
}
