use std::sync::Arc;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::webview::{PageLoadEvent, PageLoadPayload};
use tauri::{Manager, Runtime, Webview};
use xrf_renderer::RenderWorkers;

use crate::core::execution::ExecutionState;
use crate::plugins::render::packaged_bundle::PackagedBundle;
use crate::plugins::render::state::RenderState;

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

        application.manage(RenderState::new(workers, Arc::new(bundle)));

        Ok(())
      })
      .on_page_load(Self::on_page_load)
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

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::render::specta_builder()
  }
}
