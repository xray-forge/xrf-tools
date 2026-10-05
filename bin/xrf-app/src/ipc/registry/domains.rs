// Cargo compiles the application and its build script separately, so both adapters expand this token registry.
// Keep each wire name beside its Rust command path; runtime dispatch, Specta, and ACL generation derive from the pair.
// A domain's typed commands may be followed by `@raw { .. }`, then `@bulk { .. }`, in that order; either or both. Each
// entry names its TypeScript arguments, for the generated wrapper Specta cannot write, and its Rust path.
// - `@raw`: commands answering a small `tauri::ipc::Response`, dispatched and permitted like any command. No domain
//   declares one today; the section is kept supported for the next byte answer that fits a command.
// - `@bulk`: routes answering large bytes, served by the loopback transport (`core/transport/`) so they never touch the
//   window's thread; each route function takes the application and a request deserialized from those arguments.
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
      @bulk {
        read_asset(roots: "XrayRoots", logicalPath: "string") => crate::plugins::assets::routes::read_asset::assets_read_asset,
      }
      archives => "archives" {
        close_subject => crate::plugins::archives::browse::commands::close_subject::archives_close_subject,
        describe_resolution => crate::plugins::archives::browse::commands::describe_resolution::archives_describe_resolution,
        describe_statistics => crate::plugins::archives::browse::commands::describe_statistics::archives_describe_statistics,
        extract_directory => crate::plugins::archives::browse::commands::extract_directory::archives_extract_directory,
        extract_file => crate::plugins::archives::browse::commands::extract_file::archives_extract_file,
        get_subject => crate::plugins::archives::browse::commands::get_subject::archives_get_subject,
        list_overrides => crate::plugins::archives::browse::commands::list_overrides::archives_list_overrides,
        list_shared_payloads => crate::plugins::archives::browse::commands::list_shared_payloads::archives_list_shared_payloads,
        open_volumes => crate::plugins::archives::browse::commands::open_volumes::archives_open_volumes,
        open_world => crate::plugins::archives::browse::commands::open_world::archives_open_world,
        read_file => crate::plugins::archives::browse::commands::read_file::archives_read_file,
        describe_file => crate::plugins::archives::describe::commands::describe_file::archives_describe_file,
        describe_audio => crate::plugins::archives::preview::commands::describe_audio::archives_describe_audio,
        describe_texture => crate::plugins::archives::preview::commands::describe_texture::archives_describe_texture,
        describe_image => crate::plugins::archives::preview::commands::describe_image::archives_describe_image,
        default_pack_config => crate::plugins::archives::pack::commands::default_pack_config::archives_default_pack_config,
        export_pack_config => crate::plugins::archives::pack::commands::export_pack_config::archives_export_pack_config,
        import_pack_config => crate::plugins::archives::pack::commands::import_pack_config::archives_import_pack_config,
        list_pack_volumes => crate::plugins::archives::pack::commands::list_pack_volumes::archives_list_pack_volumes,
        pack_directory => crate::plugins::archives::pack::commands::pack_directory::archives_pack_directory,
        compare_archives => crate::plugins::archives::patch::commands::compare_archives::archives_compare_archives,
        default_patch_config => crate::plugins::archives::patch::commands::default_patch_config::archives_default_patch_config,
        export_patch_config => crate::plugins::archives::patch::commands::export_patch_config::archives_export_patch_config,
        import_patch_config => crate::plugins::archives::patch::commands::import_patch_config::archives_import_patch_config,
        list_patch_volumes => crate::plugins::archives::patch::commands::list_patch_volumes::archives_list_patch_volumes,
        patch_archives => crate::plugins::archives::patch::commands::patch_archives::archives_patch_archives,
        unpack_directory => crate::plugins::archives::unpack::commands::unpack_directory::archives_unpack_directory,
      }
      // Serves a decoded PNG rather than the stored DDS, so it stays here instead of joining the generic reads.
      @bulk {
        read_texture(roots: "XrayRoots", logicalPath: "string") => crate::plugins::archives::preview::routes::read_texture::archives_read_texture,
      }
      configs => "configs" {
        check_directory_format => crate::plugins::configs::commands::check_directory_format::configs_check_directory_format,
        close_project => crate::plugins::configs::commands::close_project::configs_close_project,
        format_directory => crate::plugins::configs::commands::format_directory::configs_format_directory,
        get_project => crate::plugins::configs::commands::get_project::configs_get_project,
        list_findings => crate::plugins::configs::commands::list_findings::configs_list_findings,
        list_resolved_sections => crate::plugins::configs::commands::list_resolved_sections::configs_list_resolved_sections,
        open_project => crate::plugins::configs::commands::open_project::configs_open_project,
        read_document => crate::plugins::configs::commands::read_document::configs_read_document,
        read_resolved_sections => crate::plugins::configs::commands::read_resolved_sections::configs_read_resolved_sections,
        read_section_scheme => crate::plugins::configs::commands::read_section_scheme::configs_read_section_scheme,
        verify_directory => crate::plugins::configs::commands::verify_directory::configs_verify_directory,
      }
      dialogs => "dialogs" {
        close_project => crate::plugins::dialogs::commands::close_project::dialogs_close_project,
        detect_mode => crate::plugins::dialogs::commands::detect_mode::dialogs_detect_mode,
        get_dialog => crate::plugins::dialogs::commands::get_dialog::dialogs_get_dialog,
        get_project => crate::plugins::dialogs::commands::get_project::dialogs_get_project,
        open_project => crate::plugins::dialogs::commands::open_project::dialogs_open_project,
      }
      environment => "environment" {
        read_catalog => crate::plugins::environment::commands::read_catalog::environment_read_catalog,
        read_cycle => crate::plugins::environment::commands::read_cycle::environment_read_cycle,
      }
      exports => "exports" {
        close_project => crate::plugins::exports::commands::close_project::exports_close_project,
        export_manifest => crate::plugins::exports::commands::export_manifest::exports_export_manifest,
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
      levels => "levels" {
        close_level => crate::plugins::levels::commands::close_level::levels_close_level,
        describe_console_defaults => crate::plugins::levels::commands::describe_console_defaults::levels_describe_console_defaults,
        describe_spawn_object => crate::plugins::levels::commands::describe_spawn_object::levels_describe_spawn_object,
        get_level => crate::plugins::levels::commands::get_level::levels_get_level,
        list_levels => crate::plugins::levels::commands::list_levels::levels_list_levels,
        open_level => crate::plugins::levels::commands::open_level::levels_open_level,
        open_spawn_objects => crate::plugins::levels::commands::open_spawn_objects::levels_open_spawn_objects,
        read_level_cycle => crate::plugins::levels::commands::read_level_cycle::levels_read_level_cycle,
        read_level_weather => crate::plugins::levels::commands::read_level_weather::levels_read_level_weather,
      }
      // Native viewports drawn by the renderer into the calling window, under its webview.
      render => "render" {
        attach_viewport => crate::plugins::render::commands::attach_viewport::render_attach_viewport,
        command_camera => crate::plugins::render::commands::command_camera::render_command_camera,
        configure => crate::plugins::render::commands::configure::render_configure,
        describe_frame => crate::plugins::render::commands::describe_frame::render_describe_frame,
        describe_load => crate::plugins::render::commands::describe_load::render_describe_load,
        describe_problems => crate::plugins::render::commands::describe_problems::render_describe_problems,
        describe_textures => crate::plugins::render::commands::describe_textures::render_describe_textures,
        detach_viewport => crate::plugins::render::commands::detach_viewport::render_detach_viewport,
        locate_spawn_object => crate::plugins::render::commands::locate_spawn_object::render_locate_spawn_object,
        measure_surfaces => crate::plugins::render::commands::measure_surfaces::render_measure_surfaces,
        pick => crate::plugins::render::commands::pick::render_pick,
        pose_model => crate::plugins::render::commands::pose_model::render_pose_model,
        play_ambient_effect => crate::plugins::render::commands::play_ambient_effect::render_play_ambient_effect,
        play_weather => crate::plugins::render::commands::play_weather::render_play_weather,
        play_weather_effect => crate::plugins::render::commands::play_weather_effect::render_play_weather_effect,
        save_capture => crate::plugins::render::commands::save_capture::render_save_capture,
        seek_weather => crate::plugins::render::commands::seek_weather::render_seek_weather,
        send_input => crate::plugins::render::commands::send_input::render_send_input,
        set_camera => crate::plugins::render::commands::set_camera::render_set_camera,
        set_overlays => crate::plugins::render::commands::set_overlays::render_set_overlays,
        set_selection => crate::plugins::render::commands::set_selection::render_set_selection,
        set_view_options => crate::plugins::render::commands::set_view_options::render_set_view_options,
        set_viewport_layout => crate::plugins::render::commands::set_viewport_layout::render_set_viewport_layout,
        set_weather_control => crate::plugins::render::commands::set_weather_control::render_set_weather_control,
        set_world_toggles => crate::plugins::render::commands::set_world_toggles::render_set_world_toggles,
        show_level => crate::plugins::render::commands::show_level::render_show_level,
        show_model => crate::plugins::render::commands::show_model::render_show_model,
        show_texture => crate::plugins::render::commands::show_texture::render_show_texture,
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
        get_session => crate::plugins::spawn::commands::get_session::spawn_get_session,
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
        describe_path => crate::plugins::system::paths::commands::describe_path::system_describe_path,
        get_build_info => crate::plugins::system::diagnostics::commands::get_build_info::system_get_build_info,
        get_default_output_root => crate::plugins::system::paths::commands::get_default_output_root::system_get_default_output_root,
        get_host_info => crate::plugins::system::diagnostics::commands::get_host_info::system_get_host_info,
        get_memory_usage => crate::plugins::system::diagnostics::commands::get_memory_usage::system_get_memory_usage,
        get_runtime_snapshot => crate::plugins::system::diagnostics::commands::get_runtime_snapshot::system_get_runtime_snapshot,
        reveal_path => crate::plugins::system::paths::commands::reveal_path::system_reveal_path,
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
      // Decoded images the page shows: a texture or a save candidate as png, and texels; stored bytes are `assets/read_asset`.
      @bulk {
        read_candidate(sessionId: "SessionId", format: "TextureEncodingFormat") => crate::plugins::textures::routes::read_candidate::textures_read_candidate,
        read_texels(roots: "XrayRoots", logicalPath: "string") => crate::plugins::textures::routes::read_texels::textures_read_texels,
        read_texture(roots: "XrayRoots", logicalPath: "string") => crate::plugins::textures::routes::read_texture::textures_read_texture,
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
      // Where the loopback transport listens; the `@bulk` routes are served there rather than over IPC.
      transport => "transport" {
        get_endpoint => crate::plugins::transport::commands::get_endpoint::transport_get_endpoint,
      }
    }
  };
}
