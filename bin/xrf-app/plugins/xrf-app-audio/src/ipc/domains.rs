// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `ipc` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::ipc` describes the sections.
macro_rules! for_each_audio_domain {
  ($consumer:ident) => {
    $consumer! {
      // The application's sound: its settings and output device, and the previews a domain opens for the page.
      audio => "audio" {
        close_preview => crate::commands::preview::close_preview::audio_close_preview,
        describe_output => crate::commands::settings::describe_output::audio_describe_output,
        describe_settings => crate::commands::settings::describe_settings::audio_describe_settings,
        list_devices => crate::commands::settings::list_devices::audio_list_devices,
        pause_preview => crate::commands::preview::pause_preview::audio_pause_preview,
        play_preview => crate::commands::preview::play_preview::audio_play_preview,
        read_preview_peaks => crate::commands::preview::read_preview_peaks::audio_read_preview_peaks,
        seek_preview => crate::commands::preview::seek_preview::audio_seek_preview,
        set_preview_looped => crate::commands::preview::set_preview_looped::audio_set_preview_looped,
        set_settings => crate::commands::settings::set_settings::audio_set_settings,
        stop_preview => crate::commands::preview::stop_preview::audio_stop_preview,
        watch_preview => crate::commands::preview::watch_preview::audio_watch_preview,
      }
    }
  };
}
