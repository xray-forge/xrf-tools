use std::sync::Arc;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::webview::{PageLoadEvent, PageLoadPayload};
use tauri::{AppHandle, Manager, RunEvent, Runtime, Webview, WindowEvent};
use xrf_renderer::RenderWorkers;

use crate::core::execution::ExecutionState;
use crate::plugins::render::disk_cache_store::DiskCacheStore;
use crate::plugins::render::packaged_bundle::PackagedBundle;
use crate::plugins::render::state::RenderState;

/// The `render` plugin: the native renderer's state (`RenderState`, its loaders on the application's pool, its bundle and
/// the driver's pipeline cache on disk), every `render|*` command, and the hooks that detach a window's viewports as a
/// page starts loading over them or the window is destroyed.
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

  /// Detaches a destroyed window's viewports, whose page went with it before it could let them go.
  fn on_event<R: Runtime>(application: &AppHandle<R>, event: &RunEvent) {
    if let RunEvent::WindowEvent {
      label,
      event: WindowEvent::Destroyed,
      ..
    } = event
      && let Some(state) = application.try_state::<RenderState>()
    {
      state.detach_window(label);
    }
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::render::specta_builder()
  }
}
