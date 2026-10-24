// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `registry` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::registry` describes the sections.
macro_rules! for_each_spawn_domain {
  ($consumer:ident) => {
    $consumer! {
      spawn => "spawn" {
        save_unpacked_directory => crate::commands::save_unpacked_directory::spawn_save_unpacked_directory,
        close_file => crate::commands::close_file::spawn_close_file,
        get_file => crate::commands::get_file::spawn_get_file,
        get_alife_spawns => crate::commands::get_alife_spawns::spawn_get_alife_spawns,
        get_artefact_spawns => crate::commands::get_artefact_spawns::spawn_get_artefact_spawns,
        get_graphs => crate::commands::get_graphs::spawn_get_graphs,
        get_header => crate::commands::get_header::spawn_get_header,
        get_patrols => crate::commands::get_patrols::spawn_get_patrols,
        get_session => crate::commands::get_session::spawn_get_session,
        open_unpacked_directory => crate::commands::open_unpacked_directory::spawn_open_unpacked_directory,
        open_file => crate::commands::open_file::spawn_open_file,
        pack_file => crate::commands::pack_file::spawn_pack_file,
        save_file => crate::commands::save_file::spawn_save_file,
        unpack_file => crate::commands::unpack_file::spawn_unpack_file,
      }
    }
  };
}
