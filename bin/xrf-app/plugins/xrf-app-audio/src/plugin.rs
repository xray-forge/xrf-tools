use std::path::{Path, PathBuf};

use tauri::plugin::TauriPlugin;
use tauri::webview::{PageLoadEvent, PageLoadPayload};
use tauri::{AppHandle, Manager, RunEvent, Runtime, Webview, WindowEvent};
use xrf_audio::{AudioAcoustics, AudioEngineSettings, AudioServer};

use crate::focus::AudioFocus;
use crate::preview::{PreviewEvents, PreviewTransports};
use crate::settings::AudioSettingsStore;

/// The `audio` plugin: the application's one `xrf_audio::AudioServer`, whose control thread owns the engine on the
/// output device, its acoustics on the Steam Audio the application ships; the settings it plays with, kept between runs
/// (`AudioSettingsStore`); the windows' focus and the background mute (`AudioFocus`); and the previews a domain opens
/// for the page to play (`PreviewTransports`). It knows no domain: each plays its own sounds through these units. Its
/// hooks close a window's previews as its page loads again or the window goes, follow each window's focus, and shut the
/// server down as the application exits.
pub struct AudioPlugin;

impl AudioPlugin {
  /// Where a packaged application keeps Steam Audio in its resources, as `tauri.windows.conf.json` and
  /// `tauri.macos.conf.json` map it.
  const PACKAGED_ACOUSTICS: &'static str = "resources/audio/steam-audio";

  /// Where `cargo make fetch-sdks` places it in the checkout, which a development build loads it from.
  const CHECKOUT_ACOUSTICS: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../.sdks/steam-audio");

  /// The plugin, its state managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::audio::builder()
      .setup(|application, _| {
        let store: AudioSettingsStore = AudioSettingsStore::open(application);
        let events: PreviewEvents = PreviewEvents::default();
        let acoustics: AudioAcoustics = AudioAcoustics::load(&Self::locate_acoustics(
          application.path().resource_dir().ok().as_deref(),
        ));
        let server: AudioServer = AudioServer::start(
          AudioEngineSettings::default(),
          store.get().output,
          acoustics,
          events.clone(),
        )?;
        let focus: AudioFocus = AudioFocus::default();

        server.set_spatialization(store.get().spatialization);
        focus.apply(&server, &store);
        application.manage(PreviewTransports::new(server.clone(), &events));
        application.manage(server);
        application.manage(focus);
        application.manage(store);

        Ok(())
      })
      .on_page_load(Self::on_page_load)
      .on_event(Self::on_event)
      .build()
  }

  /// The directory Steam Audio's library is loaded from: the packaged resources', or the checkout's where nothing was
  /// packaged, as a development build runs.
  fn locate_acoustics(resource_dir: Option<&Path>) -> PathBuf {
    resource_dir
      .map(|dir| dir.join(Self::PACKAGED_ACOUSTICS))
      .filter(|dir| dir.join(AudioAcoustics::LIBRARY).is_file())
      .unwrap_or_else(|| PathBuf::from(Self::CHECKOUT_ACOUSTICS))
  }

  /// Closes a window's previews as its page starts loading, since the page that opened them can no longer close them.
  fn on_page_load<R: Runtime>(webview: &Webview<R>, payload: &PageLoadPayload<'_>) {
    if payload.event() == PageLoadEvent::Started
      && let Some(transports) = webview.try_state::<PreviewTransports>()
    {
      transports.close_window(webview.window().label());
    }
  }

  fn on_event<R: Runtime>(application: &AppHandle<R>, event: &RunEvent) {
    match event {
      RunEvent::WindowEvent {
        label,
        event: WindowEvent::Focused(is_focused),
        ..
      } => Self::follow_focus(application, label, *is_focused),
      RunEvent::WindowEvent {
        label,
        event: WindowEvent::Destroyed,
        ..
      } => {
        Self::follow_focus(application, label, false);

        if let Some(transports) = application.try_state::<PreviewTransports>() {
          transports.close_window(label);
        }
      }
      RunEvent::Exit => {
        if let Some(transports) = application.try_state::<PreviewTransports>() {
          transports.shut_down();
        }
      }
      _ => {}
    }
  }

  /// Notes the window's focus, and moves the gains when the application went to or came back from the background.
  fn follow_focus<R: Runtime>(application: &AppHandle<R>, label: &str, is_focused: bool) {
    let (Some(focus), Some(server), Some(store)) = (
      application.try_state::<AudioFocus>(),
      application.try_state::<AudioServer>(),
      application.try_state::<AudioSettingsStore>(),
    ) else {
      return;
    };

    if focus.set_window_focused(label, is_focused) {
      focus.apply(&server, &store);
    }
  }
}
