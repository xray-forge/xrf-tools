// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `registry` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::registry` describes the sections.
macro_rules! for_each_configs_domain {
  ($consumer:ident) => {
    $consumer! {
      configs => "configs" {
        check_directory_format => crate::commands::check_directory_format::configs_check_directory_format,
        close_project => crate::commands::close_project::configs_close_project,
        format_directory => crate::commands::format_directory::configs_format_directory,
        get_project => crate::commands::get_project::configs_get_project,
        list_findings => crate::commands::list_findings::configs_list_findings,
        list_resolved_sections => crate::commands::list_resolved_sections::configs_list_resolved_sections,
        open_project => crate::commands::open_project::configs_open_project,
        read_document => crate::commands::read_document::configs_read_document,
        read_resolved_sections => crate::commands::read_resolved_sections::configs_read_resolved_sections,
        read_section_scheme => crate::commands::read_section_scheme::configs_read_section_scheme,
        verify_directory => crate::commands::verify_directory::configs_verify_directory,
      }
    }
  };
}
