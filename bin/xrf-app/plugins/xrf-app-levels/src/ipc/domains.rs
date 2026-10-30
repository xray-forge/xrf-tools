// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `ipc` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::ipc` describes the sections.
macro_rules! for_each_levels_domain {
  ($consumer:ident) => {
    $consumer! {
      levels => "levels" {
        capture_cube => crate::viewer::commands::capture_cube::levels_capture_cube,
        capture_map => crate::viewer::commands::capture_map::levels_capture_map,
        close_level => crate::commands::close_level::levels_close_level,
        copy_screenshot => crate::viewer::commands::copy_screenshot::levels_copy_screenshot,
        delete_screenshot => crate::viewer::commands::delete_screenshot::levels_delete_screenshot,
        describe_console_defaults => crate::commands::describe_console_defaults::levels_describe_console_defaults,
        describe_level_map => crate::commands::describe_level_map::levels_describe_level_map,
        describe_objects => crate::commands::describe_objects::levels_describe_objects,
        describe_spawn_object => crate::commands::describe_spawn_object::levels_describe_spawn_object,
        describe_start => crate::commands::describe_start::levels_describe_start,
        describe_surfaces => crate::commands::describe_surfaces::levels_describe_surfaces,
        describe_textures => crate::commands::describe_textures::levels_describe_textures,
        describe_weather => crate::commands::describe_weather::levels_describe_weather,
        export_cube => crate::viewer::commands::export_cube::levels_export_cube,
        export_map => crate::viewer::commands::export_map::levels_export_map,
        find_screenshot_path => crate::viewer::commands::find_screenshot_path::levels_find_screenshot_path,
        get_view_folder => crate::viewer::commands::get_view_folder::levels_get_view_folder,
        list_levels => crate::commands::list_levels::levels_list_levels,
        open_level => crate::commands::open_level::levels_open_level,
        read_level_cycle => crate::commands::read_level_cycle::levels_read_level_cycle,
        read_view => crate::viewer::commands::read_view::levels_read_view,
        restore_level => crate::commands::restore_level::levels_restore_level,
        save_burst => crate::viewer::commands::save_burst::levels_save_burst,
        save_screenshot => crate::viewer::commands::save_screenshot::levels_save_screenshot,
        save_view => crate::viewer::commands::save_view::levels_save_view,
        show_level => crate::commands::show_level::levels_show_level,
        write_map_annotations => crate::viewer::commands::write_map_annotations::levels_write_map_annotations,
        write_screenshot_report => crate::viewer::commands::write_screenshot_report::levels_write_screenshot_report,
      }
      @bulk {
        read_screenshot(path: "string") => crate::viewer::routes::read_screenshot::levels_read_screenshot,
      }
      @jobs {
        /// Capturing a cubemap around a viewport's camera, face by face.
        LEVELS_JOB_CAPTURE_CUBE => "capture-cube",
        /// Capturing a level map in a viewport, tile by tile.
        LEVELS_JOB_CAPTURE_MAP => "capture-map",
      }
    }
  };
}
