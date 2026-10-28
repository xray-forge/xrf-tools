// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `ipc` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::ipc` describes the sections.
macro_rules! for_each_render_domain {
  ($consumer:ident) => {
    $consumer! {
      // Native viewports drawn by the renderer into the window the page names, under its webview.
      render => "render" {
        attach_viewport => crate::commands::attach_viewport::render_attach_viewport,
        command_camera => crate::commands::command_camera::render_command_camera,
        configure => crate::commands::configure::render_configure,
        describe_frame => crate::commands::describe_frame::render_describe_frame,
        describe_libraries => crate::commands::describe_libraries::render_describe_libraries,
        describe_load => crate::commands::describe_load::render_describe_load,
        describe_problems => crate::commands::describe_problems::render_describe_problems,
        describe_texture_loads => crate::commands::describe_texture_loads::render_describe_texture_loads,
        detach_viewport => crate::commands::detach_viewport::render_detach_viewport,
        list_backends => crate::commands::list_backends::render_list_backends,
        locate_spawn_object => crate::commands::locate_spawn_object::render_locate_spawn_object,
        mark_profile => crate::commands::mark_profile::render_mark_profile,
        measure_surfaces => crate::commands::measure_surfaces::render_measure_surfaces,
        pick => crate::commands::pick::render_pick,
        pose_model => crate::commands::pose_model::render_pose_model,
        play_ambient_effect => crate::commands::play_ambient_effect::render_play_ambient_effect,
        play_weather => crate::commands::play_weather::render_play_weather,
        play_weather_effect => crate::commands::play_weather_effect::render_play_weather_effect,
        save_capture => crate::commands::save_capture::render_save_capture,
        seek_weather => crate::commands::seek_weather::render_seek_weather,
        send_input => crate::commands::send_input::render_send_input,
        set_camera => crate::commands::set_camera::render_set_camera,
        set_overlays => crate::commands::set_overlays::render_set_overlays,
        set_selection => crate::commands::set_selection::render_set_selection,
        set_view_options => crate::commands::set_view_options::render_set_view_options,
        set_viewport_layout => crate::commands::set_viewport_layout::render_set_viewport_layout,
        set_weather_control => crate::commands::set_weather_control::render_set_weather_control,
        set_world_toggles => crate::commands::set_world_toggles::render_set_world_toggles,
        step_clock => crate::commands::step_clock::render_step_clock,
      }
    }
  };
}
