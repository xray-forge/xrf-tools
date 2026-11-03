use tauri::plugin::TauriPlugin;
use tauri::webview::{PageLoadEvent, PageLoadPayload};
use tauri::{AppHandle, Manager, RunEvent, Runtime, Webview, WindowEvent};
use xrf_audio::{AudioEngineSettings, AudioServer};

use crate::focus::AudioFocus;
use crate::preview::{PreviewEvents, PreviewTransports};
use crate::settings::AudioSettingsStore;

/// The `audio` plugin: the application's one `xrf_audio::AudioServer`, whose control thread owns the engine on the
/// output device; the settings it plays with, kept between runs (`AudioSettingsStore`); the windows' focus and the
/// background mute (`AudioFocus`); and the previews a domain opens for the page to play (`PreviewTransports`). It knows
/// no domain: each plays its own sounds through these units. Its hooks close a window's previews as its page loads
/// again or the window goes, follow each window's focus, and shut the server down as the application exits.
pub struct AudioPlugin;

impl AudioPlugin {
  /// The plugin, its state managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::audio::builder()
      .setup(|application, _| {
        let store: AudioSettingsStore = AudioSettingsStore::open(application);
        let events: PreviewEvents = PreviewEvents::default();
        let server: AudioServer =
          AudioServer::start(AudioEngineSettings::default(), store.get().output, events.clone())?;
        let focus: AudioFocus = AudioFocus::default();

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
