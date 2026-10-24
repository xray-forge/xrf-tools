// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `registry` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::registry` describes the sections.
macro_rules! for_each_translations_domain {
  ($consumer:ident) => {
    $consumer! {
      translations => "translations" {
        build_project => crate::commands::build_project::translations_build_project,
        check_project_format => crate::commands::check_project_format::translations_check_project_format,
        close_project => crate::commands::close_project::translations_close_project,
        detect_mode => crate::commands::detect_mode::translations_detect_mode,
        format_project => crate::commands::format_project::translations_format_project,
        get_project => crate::commands::get_project::translations_get_project,
        open_project => crate::commands::open_project::translations_open_project,
        parse_project => crate::commands::parse_project::translations_parse_project,
        save_file => crate::commands::save_file::translations_save_file,
        validate_text => crate::commands::validate_text::translations_validate_text,
        verify_project => crate::commands::verify_project::translations_verify_project,
      }
    }
  };
}
