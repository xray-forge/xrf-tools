// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `ipc` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::ipc` describes the sections.
macro_rules! for_each_spawn_domain {
  ($consumer:ident) => {
    $consumer! {
      spawn => "spawn" {
        close_file => crate::commands::close_file::spawn_close_file,
        open_file => crate::commands::open_file::spawn_open_file,
        pack_file => crate::commands::pack_file::spawn_pack_file,
        read_chunk => crate::commands::read_chunk::spawn_read_chunk,
        restore_file => crate::commands::restore_file::spawn_restore_file,
        save_file => crate::commands::save_file::spawn_save_file,
        save_unpacked_directory => crate::commands::save_unpacked_directory::spawn_save_unpacked_directory,
        unpack_file => crate::commands::unpack_file::spawn_unpack_file,
      }
      @jobs {
        /// Packing a directory of spawn chunks into one file.
        SPAWN_JOB_PACK => "pack",
        /// Unpacking a spawn file into a directory of chunks.
        SPAWN_JOB_UNPACK => "unpack",
      }
    }
  };
}
