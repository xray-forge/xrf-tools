use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};
use xrf_renderer::RenderWorkers;

use crate::core::execution::ExecutionState;
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

        application.manage(RenderState::new(workers));

        Ok(())
      })
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::render::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::render::specta_builder()
  }
}
