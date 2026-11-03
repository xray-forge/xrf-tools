use tauri::plugin::TauriPlugin;
use tauri::webview::{PageLoadEvent, PageLoadPayload};
use tauri::{AppHandle, Manager, RunEvent, Runtime, Webview, WindowEvent};

use crate::engine::AudioEngineState;
use crate::preview::PreviewTransports;
use crate::settings::AudioSettingsStore;

/// The `audio` plugin: the application's one audio engine on its output device (`AudioEngineState`), the settings it
/// plays with, kept between runs (`AudioSettingsStore`), and the previews a domain opens for the page to play
/// (`PreviewTransports`). It knows no domain: each plays its own sounds through these units. Its hooks close a window's
/// previews as its page loads again or the window goes, mute the application where the settings say so once no window
/// has the focus, and close every preview as the application exits.
pub struct AudioPlugin;

impl AudioPlugin {
  /// The plugin, its state managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::audio::builder()
      .setup(|application, _| {
        let store: AudioSettingsStore = AudioSettingsStore::open(application);
        let engine: AudioEngineState = AudioEngineState::new(store.get());

        application.manage(PreviewTransports::start(engine.clone()));
        application.manage(engine);
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
      } => {
        if let Some(engine) = application.try_state::<AudioEngineState>() {
          engine.set_window_focused(label, *is_focused);
        }
      }
      RunEvent::WindowEvent {
        label,
        event: WindowEvent::Destroyed,
        ..
      } => {
        if let Some(engine) = application.try_state::<AudioEngineState>() {
          engine.set_window_focused(label, false);
        }

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
}
