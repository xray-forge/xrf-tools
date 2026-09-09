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
        probe_root => crate::plugins::assets::commands::probe_root::assets_probe_root,
      }
      @raw {
        read_asset(roots: "XrayRoots", logicalPath: "string") => crate::plugins::assets::commands::read_asset::assets_read_asset,
      }
      archives => "archives" {
        close_project => crate::plugins::archives::commands::close_project::archives_close_project,
        compare_archives => crate::plugins::archives::commands::compare_archives::archives_compare_archives,
        default_pack_config => crate::plugins::archives::commands::default_pack_config::archives_default_pack_config,
        default_patch_config => crate::plugins::archives::commands::default_patch_config::archives_default_patch_config,
        describe_audio => crate::plugins::archives::commands::describe_audio::archives_describe_audio,
        describe_image => crate::plugins::archives::commands::describe_image::archives_describe_image,
        export_pack_config => crate::plugins::archives::commands::export_pack_config::archives_export_pack_config,
        export_patch_config => crate::plugins::archives::commands::export_patch_config::archives_export_patch_config,
        import_pack_config => crate::plugins::archives::commands::import_pack_config::archives_import_pack_config,
        import_patch_config => crate::plugins::archives::commands::import_patch_config::archives_import_patch_config,
        extract_file => crate::plugins::archives::commands::extract_file::archives_extract_file,
        extract_directory => crate::plugins::archives::commands::extract_directory::archives_extract_directory,
        get_project => crate::plugins::archives::commands::get_project::archives_get_project,
        has_project => crate::plugins::archives::commands::has_project::archives_has_project,
        list_collisions => crate::plugins::archives::commands::list_collisions::archives_list_collisions,
        list_pack_volumes => crate::plugins::archives::commands::list_pack_volumes::archives_list_pack_volumes,
        list_patch_volumes => crate::plugins::archives::commands::list_patch_volumes::archives_list_patch_volumes,
        list_shared_payloads => crate::plugins::archives::commands::list_shared_payloads::archives_list_shared_payloads,
        open_project => crate::plugins::archives::commands::open_project::archives_open_project,
        pack_directory => crate::plugins::archives::commands::pack_directory::archives_pack_directory,
        patch_archives => crate::plugins::archives::commands::patch_archives::archives_patch_archives,
        read_file => crate::plugins::archives::commands::read_file::archives_read_file,
        unpack_directory => crate::plugins::archives::commands::unpack_directory::archives_unpack_directory,
      }
      // Serves a decoded PNG rather than the stored DDS, so it stays here instead of joining the generic reads.
      @raw {
        read_image(roots: "XrayRoots", logicalPath: "string") => crate::plugins::archives::commands::read_image::archives_read_image,
      }
      configs => "configs" {
        check_directory_format => crate::plugins::configs::commands::check_directory_format::configs_check_directory_format,
        close_project => crate::plugins::configs::commands::close_project::configs_close_project,
        format_directory => crate::plugins::configs::commands::format_directory::configs_format_directory,
        get_project => crate::plugins::configs::commands::get_project::configs_get_project,
        list_resolved_sections => crate::plugins::configs::commands::list_resolved_sections::configs_list_resolved_sections,
        open_project => crate::plugins::configs::commands::open_project::configs_open_project,
        read_document => crate::plugins::configs::commands::read_document::configs_read_document,
        read_resolved_sections => crate::plugins::configs::commands::read_resolved_sections::configs_read_resolved_sections,
        verify_directory => crate::plugins::configs::commands::verify_directory::configs_verify_directory,
      }
      dialogs => "dialogs" {
        close_project => crate::plugins::dialogs::commands::close_project::dialogs_close_project,
        detect_mode => crate::plugins::dialogs::commands::detect_mode::dialogs_detect_mode,
        get_dialog => crate::plugins::dialogs::commands::get_dialog::dialogs_get_dialog,
        get_project => crate::plugins::dialogs::commands::get_project::dialogs_get_project,
        open_project => crate::plugins::dialogs::commands::open_project::dialogs_open_project,
      }
      exports => "exports" {
        close_project => crate::plugins::exports::commands::close_project::exports_close_project,
        open_project => crate::plugins::exports::commands::open_project::exports_open_project,
        get_project => crate::plugins::exports::commands::get_project::exports_get_project,
        get_source => crate::plugins::exports::commands::get_source::exports_get_source,
      }
      // Running work, whatever domain it belongs to: what is going on, and asking it to stop. A domain of its own
      // because identity, exclusion and cancellation are the same questions for a pack, a verification, or a build.
      gamedata => "gamedata" {
        verify_project => crate::plugins::gamedata::commands::verify_project::gamedata_verify_project,
      }
      jobs => "jobs" {
        attach => crate::plugins::jobs::commands::attach::jobs_attach,
        cancel => crate::plugins::jobs::commands::cancel::jobs_cancel,
        list => crate::plugins::jobs::commands::list::jobs_list,
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
      sprite_equipment => "sprite-equipment" {
        close_sprite => crate::plugins::sprite_equipment::commands::close_sprite::sprite_equipment_close_sprite,
        get_sprite => crate::plugins::sprite_equipment::commands::get_sprite::sprite_equipment_get_sprite,
        open_sprite => crate::plugins::sprite_equipment::commands::open_sprite::sprite_equipment_open_sprite,
        reopen_sprite => crate::plugins::sprite_equipment::commands::reopen_sprite::sprite_equipment_reopen_sprite,
        pack_sprite => crate::plugins::sprite_equipment::commands::pack_sprite::sprite_equipment_pack_sprite,
      }
      system => "system" {
        describe_path => crate::plugins::system::commands::describe_path::system_describe_path,
        get_build_info => crate::plugins::system::commands::get_build_info::system_get_build_info,
        get_default_output_root => crate::plugins::system::commands::get_default_output_root::system_get_default_output_root,
        reveal_path => crate::plugins::system::commands::reveal_path::system_reveal_path,
      }
      textures => "textures" {
        build_from_source => crate::plugins::textures::commands::build_from_source::textures_build_from_source,
        close => crate::plugins::textures::commands::close::textures_close,
        compare_encodings => crate::plugins::textures::commands::compare_encodings::textures_compare_encodings,
        describe => crate::plugins::textures::commands::describe::textures_describe,
        describe_catalog => crate::plugins::textures::commands::describe_catalog::textures_describe_catalog,
        get_session => crate::plugins::textures::commands::get_session::textures_get_session,
        get_vocabulary => crate::plugins::textures::commands::get_vocabulary::textures_get_vocabulary,
        make_bump => crate::plugins::textures::commands::make_bump::textures_make_bump,
        open => crate::plugins::textures::commands::open::textures_open,
        save => crate::plugins::textures::commands::save::textures_save,
      }
      // The png fallback for a layout the webview's DDS loader refuses; stored bytes go through `assets|read_asset`.
      @raw {
        read_candidate(sessionId: "TextureSessionId", format: "TextureEncodingFormat") => crate::plugins::textures::commands::read_candidate::textures_read_candidate,
        read_texture(roots: "XrayRoots", logicalPath: "string") => crate::plugins::textures::commands::read_texture::textures_read_texture,
      }
      visuals => "visuals" {
        close_browse => crate::plugins::visuals::commands::close_browse::visuals_close_browse,
        close_model => crate::plugins::visuals::commands::close_model::visuals_close_model,
        get_browse => crate::plugins::visuals::commands::get_browse::visuals_get_browse,
        get_model => crate::plugins::visuals::commands::get_model::visuals_get_model,
        list_motions => crate::plugins::visuals::commands::list_motions::visuals_list_motions,
        open_browse => crate::plugins::visuals::commands::open_browse::visuals_open_browse,
        open_model => crate::plugins::visuals::commands::open_model::visuals_open_model,
        open_motion => crate::plugins::visuals::commands::open_motion::visuals_open_motion,
      }
      // Returns `tauri::ipc::Response`, so it is dispatched and permitted like any command but cannot join
      // the Specta collection.
      @raw {
        read_geometry(source: "VisualSource", roots: "XrayRoots") => crate::plugins::visuals::commands::read_geometry::visuals_read_geometry,
        read_motion(name: "string") => crate::plugins::visuals::commands::read_motion::visuals_read_motion,
        read_texture(roots: "XrayRoots", logicalPath: "string") => crate::plugins::visuals::commands::read_texture::visuals_read_texture,
      }
      translations => "translations" {
        build_project => crate::plugins::translations::commands::build_project::translations_build_project,
        check_project_format => crate::plugins::translations::commands::check_project_format::translations_check_project_format,
        close_project => crate::plugins::translations::commands::close_project::translations_close_project,
        detect_mode => crate::plugins::translations::commands::detect_mode::translations_detect_mode,
        format_project => crate::plugins::translations::commands::format_project::translations_format_project,
        get_project => crate::plugins::translations::commands::get_project::translations_get_project,
        open_project => crate::plugins::translations::commands::open_project::translations_open_project,
        parse_project => crate::plugins::translations::commands::parse_project::translations_parse_project,
        save_file => crate::plugins::translations::commands::save_file::translations_save_file,
        validate_text => crate::plugins::translations::commands::validate_text::translations_validate_text,
        verify_project => crate::plugins::translations::commands::verify_project::translations_verify_project,
      }
    }
  };
}
