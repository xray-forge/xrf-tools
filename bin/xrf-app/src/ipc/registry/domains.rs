// Cargo compiles the application and its build script separately, so both adapters expand this token registry.
// Keep each wire name beside its Rust command path; runtime dispatch, Specta, and ACL generation derive from the pair.
macro_rules! for_each_tauri_command_domain {
  ($consumer:ident) => {
    $consumer! {
      // Bytes of any mounted asset, for every domain: a texture of a model, an entry of an archive, a level's own tree.
      // Reading is generic, so it lives here rather than being reimplemented per domain; what an asset *means* stays with
      // the domain that parses it.
      assets => "assets" {
        list_assets => crate::plugins::assets::commands::list_assets::assets_list_assets,
      }
      @raw {
        read_asset(world: "XrayWorldSpec", logicalPath: "string") => crate::plugins::assets::commands::read_asset::assets_read_asset,
      }
      archives => "archives" {
        close_project => crate::plugins::archives::commands::close_project::archives_close_project,
        default_pack_config => crate::plugins::archives::commands::default_pack_config::archives_default_pack_config,
        describe_audio => crate::plugins::archives::commands::describe_audio::archives_describe_audio,
        describe_image => crate::plugins::archives::commands::describe_image::archives_describe_image,
        export_pack_config => crate::plugins::archives::commands::export_pack_config::archives_export_pack_config,
        import_pack_config => crate::plugins::archives::commands::import_pack_config::archives_import_pack_config,
        extract_file => crate::plugins::archives::commands::extract_file::archives_extract_file,
        extract_directory => crate::plugins::archives::commands::extract_directory::archives_extract_directory,
        get_project => crate::plugins::archives::commands::get_project::archives_get_project,
        has_project => crate::plugins::archives::commands::has_project::archives_has_project,
        open_project => crate::plugins::archives::commands::open_project::archives_open_project,
        pack_directory => crate::plugins::archives::commands::pack_directory::archives_pack_directory,
        read_file => crate::plugins::archives::commands::read_file::archives_read_file,
        unpack_directory => crate::plugins::archives::commands::unpack_directory::archives_unpack_directory,
      }
      // Serves a decoded PNG rather than the stored DDS, so it stays here instead of joining the generic reads.
      @raw {
        read_image(world: "XrayWorldSpec", logicalPath: "string") => crate::plugins::archives::commands::read_image::archives_read_image,
      }
      configs => "configs" {
        check_directory_format => crate::plugins::configs::commands::check_directory_format::configs_check_directory_format,
        format_directory => crate::plugins::configs::commands::format_directory::configs_format_directory,
        verify_directory => crate::plugins::configs::commands::verify_directory::configs_verify_directory,
      }
      dialogs => "dialogs" {
        close_project => crate::plugins::dialogs::commands::close_project::dialogs_close_project,
        detect_mode => crate::plugins::dialogs::commands::detect_mode::dialogs_detect_mode,
        get_project => crate::plugins::dialogs::commands::get_project::dialogs_get_project,
        open_project => crate::plugins::dialogs::commands::open_project::dialogs_open_project,
      }
      exports => "exports" {
        close_project => crate::plugins::exports::commands::close_project::exports_close_project,
        open_project => crate::plugins::exports::commands::open_project::exports_open_project,
        get_project => crate::plugins::exports::commands::get_project::exports_get_project,
        get_source => crate::plugins::exports::commands::get_source::exports_get_source,
      }
      equipment_icons => "equipment-icons" {
        close_sprite => crate::plugins::equipment_icons::commands::close_sprite::equipment_icons_close_sprite,
        get_sprite => crate::plugins::equipment_icons::commands::get_sprite::equipment_icons_get_sprite,
        open_sprite => crate::plugins::equipment_icons::commands::open_sprite::equipment_icons_open_sprite,
        reopen_sprite => crate::plugins::equipment_icons::commands::reopen_sprite::equipment_icons_reopen_sprite,
        pack_sprite => crate::plugins::equipment_icons::commands::pack_sprite::equipment_icons_pack_sprite,
      }
      spawn => "spawn" {
        save_unpacked_directory => crate::plugins::spawn::commands::save_unpacked_directory::spawn_save_unpacked_directory,
        close_file => crate::plugins::spawn::commands::close_file::spawn_close_file,
        get_file => crate::plugins::spawn::commands::get_file::spawn_get_file,
        get_alife_spawns => crate::plugins::spawn::commands::get_alife_spawns::spawn_get_alife_spawns,
        get_artefact_spawns => crate::plugins::spawn::commands::get_artefact_spawns::spawn_get_artefact_spawns,
        get_graphs => crate::plugins::spawn::commands::get_graphs::spawn_get_graphs,
        get_header => crate::plugins::spawn::commands::get_header::spawn_get_header,
        get_patrols => crate::plugins::spawn::commands::get_patrols::spawn_get_patrols,
        get_path => crate::plugins::spawn::commands::get_path::spawn_get_path,
        has_file => crate::plugins::spawn::commands::has_file::spawn_has_file,
        open_unpacked_directory => crate::plugins::spawn::commands::open_unpacked_directory::spawn_open_unpacked_directory,
        open_file => crate::plugins::spawn::commands::open_file::spawn_open_file,
        pack_file => crate::plugins::spawn::commands::pack_file::spawn_pack_file,
        save_file => crate::plugins::spawn::commands::save_file::spawn_save_file,
        unpack_file => crate::plugins::spawn::commands::unpack_file::spawn_unpack_file,
      }
      system => "system" {
        reveal_path => crate::plugins::system::commands::reveal_path::system_reveal_path,
      }
      visuals => "visuals" {
        close_browse => crate::plugins::visuals::commands::close_browse::visuals_close_browse,
        close_model => crate::plugins::visuals::commands::close_model::visuals_close_model,
        get_browse => crate::plugins::visuals::commands::get_browse::visuals_get_browse,
        get_model => crate::plugins::visuals::commands::get_model::visuals_get_model,
        open_browse => crate::plugins::visuals::commands::open_browse::visuals_open_browse,
        open_model => crate::plugins::visuals::commands::open_model::visuals_open_model,
      }
      // Returns `tauri::ipc::Response`, so it is dispatched and permitted like any command but cannot join
      // the Specta collection.
      @raw {
        read_geometry(source: "VisualSource", world: "XrayWorldSpec") => crate::plugins::visuals::commands::read_geometry::visuals_read_geometry,
      }
      translations => "translations" {
        close_project => crate::plugins::translations::commands::close_project::translations_close_project,
        detect_mode => crate::plugins::translations::commands::detect_mode::translations_detect_mode,
        get_project => crate::plugins::translations::commands::get_project::translations_get_project,
        open_project => crate::plugins::translations::commands::open_project::translations_open_project,
        save_file => crate::plugins::translations::commands::save_file::translations_save_file,
        validate_text => crate::plugins::translations::commands::validate_text::translations_validate_text,
      }
    }
  };
}
