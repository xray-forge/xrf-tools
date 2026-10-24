// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `registry` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::registry` describes the sections.
macro_rules! for_each_archives_domain {
  ($consumer:ident) => {
    $consumer! {
      archives => "archives" {
        close_subject => crate::browse::commands::close_subject::archives_close_subject,
        describe_resolution => crate::browse::commands::describe_resolution::archives_describe_resolution,
        describe_statistics => crate::browse::commands::describe_statistics::archives_describe_statistics,
        extract_directory => crate::browse::commands::extract_directory::archives_extract_directory,
        extract_file => crate::browse::commands::extract_file::archives_extract_file,
        get_subject => crate::browse::commands::get_subject::archives_get_subject,
        list_overrides => crate::browse::commands::list_overrides::archives_list_overrides,
        list_shared_payloads => crate::browse::commands::list_shared_payloads::archives_list_shared_payloads,
        open_volumes => crate::browse::commands::open_volumes::archives_open_volumes,
        open_world => crate::browse::commands::open_world::archives_open_world,
        read_file => crate::browse::commands::read_file::archives_read_file,
        describe_file => crate::describe::commands::describe_file::archives_describe_file,
        describe_audio => crate::preview::commands::describe_audio::archives_describe_audio,
        describe_texture => crate::preview::commands::describe_texture::archives_describe_texture,
        describe_image => crate::preview::commands::describe_image::archives_describe_image,
        default_pack_config => crate::pack::commands::default_pack_config::archives_default_pack_config,
        export_pack_config => crate::pack::commands::export_pack_config::archives_export_pack_config,
        import_pack_config => crate::pack::commands::import_pack_config::archives_import_pack_config,
        list_pack_volumes => crate::pack::commands::list_pack_volumes::archives_list_pack_volumes,
        pack_directory => crate::pack::commands::pack_directory::archives_pack_directory,
        compare_archives => crate::patch::commands::compare_archives::archives_compare_archives,
        default_patch_config => crate::patch::commands::default_patch_config::archives_default_patch_config,
        export_patch_config => crate::patch::commands::export_patch_config::archives_export_patch_config,
        import_patch_config => crate::patch::commands::import_patch_config::archives_import_patch_config,
        list_patch_volumes => crate::patch::commands::list_patch_volumes::archives_list_patch_volumes,
        patch_archives => crate::patch::commands::patch_archives::archives_patch_archives,
        unpack_directory => crate::unpack::commands::unpack_directory::archives_unpack_directory,
      }
      // Serves a decoded PNG rather than the stored DDS, so it stays here instead of joining the generic reads.
      @bulk {
        read_texture(roots: "XrayRoots", logicalPath: "string") => crate::preview::routes::read_texture::archives_read_texture,
      }
    }
  };
}
