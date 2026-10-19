use std::sync::Arc;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::webview::{PageLoadEvent, PageLoadPayload, WebviewWindow};
use tauri::{AppHandle, Manager, RunEvent, Runtime, Webview, WindowEvent};
use xrf_renderer::RenderWorkers;

use crate::core::execution::ExecutionState;
use crate::core::window::{WindowCloseApprovals, WindowHandles};
use crate::plugins::render::disk_cache_store::DiskCacheStore;
use crate::plugins::render::packaged_bundle::PackagedBundle;
use crate::plugins::render::state::RenderState;

/// The `render` plugin: the native renderer's state (`RenderState`, its loaders on the application's pool, its bundle and
/// the driver's pipeline cache on disk), every `render|*` command, and the hooks that detach a window's viewports as a
/// page starts loading over them or the window closes, and stop the renderer as the application exits.
pub struct RenderPlugin {}

impl RenderPlugin {
  pub const NAME: &'static str = crate::ipc::registry::render::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    Builder::new(Self::NAME)
      .setup(|application, _| {
        // The renderer's loaders run on the application's one pool, beside every other job.
        let workers: RenderWorkers = RenderWorkers::new(application.state::<ExecutionState>().get_pool());
        let bundle: PackagedBundle = PackagedBundle::locate(application.path().resource_dir().ok().as_deref());
        // The driver's compiled pipelines, kept between runs where the backend keeps them.
        let store: Option<DiskCacheStore> = application
          .path()
          .app_cache_dir()
          .ok()
          .map(|directory| DiskCacheStore::new(directory.join("renderer")));

        application.manage(RenderState::new(workers, Arc::new(bundle), store));

        Ok(())
      })
      .on_page_load(Self::on_page_load)
      .on_event(Self::on_event)
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::render::handler(),
      ))
      .build()
  }

  /// Detaches a window's viewports as a page starts loading over it, since their channels never learn it went.
  fn on_page_load<R: Runtime>(webview: &Webview<R>, payload: &PageLoadPayload<'_>) {
    if payload.event() != PageLoadEvent::Started {
      return;
    }

    if let Some(state) = webview.try_state::<RenderState>() {
      state.detach_window(webview.window().label());
    }
  }

  /// Detaches a closing window's viewports, whose page goes with it before it can let them go, holding the window open
  /// until the renderer let go of it and destroying it then; forgets a destroyed window; stops the renderer as the
  /// application exits. A close the page has not approved is the page's to decide, so it is left alone.
  fn on_event<R: Runtime>(application: &AppHandle<R>, event: &RunEvent) {
    let Some(state) = application.try_state::<RenderState>() else {
      return;
    };

    match event {
      RunEvent::WindowEvent {
        label,
        event: WindowEvent::CloseRequested { api, .. },
        ..
      } => {
        let is_approved: bool = application
          .try_state::<WindowCloseApprovals>()
          .is_some_and(|approvals| approvals.take(label));

        // Asked behind the page's prompt: the page answers, and asks again once it agrees. A window already held stays
        // held whoever asks.
        if !is_approved && !state.is_closing(label) {
          return;
        }

        // A window on its way out hosts no viewport attached from now on.
        if let Some(handles) = application.try_state::<WindowHandles>() {
          handles.forget(label);
        }

        let window: Option<WebviewWindow<R>> = application.get_webview_window(label);
        let is_held: bool = state.close_window(label, move || {
          if let Some(window) = window
            && let Err(error) = window.destroy()
          {
            log::error!("Window '{}' cannot be closed: {error}", window.label());
          }
        });

        if is_held {
          api.prevent_close();
        }
      }
      RunEvent::WindowEvent {
        label,
        event: WindowEvent::Destroyed,
        ..
      } => {
        state.forget_window(label);

        if let Some(handles) = application.try_state::<WindowHandles>() {
          handles.forget(label);
        }
      }
      RunEvent::Exit => state.shut_down(),
      _ => {}
    }
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::render::specta_builder()
  }
}
