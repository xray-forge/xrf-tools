use std::num::NonZeroIsize;
use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use xrf_app_core::TauriResult;
use xrf_app_core::window::{InvokingWindow, WindowHandles};
use xrf_renderer::{RenderViewportId, RenderWindowHost};

use crate::window_host::WindowHostBuilder;
use crate::{RenderViewports, ViewportEvent};

/// Start drawing a native viewport into the calling window, telling the page what it costs and where its camera is
/// through `events`.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "attach_viewport"))]
#[tauri::command(rename = "attach_viewport")]
pub fn render_attach_viewport(
  viewports: State<'_, RenderViewports>,
  handles: State<'_, WindowHandles>,
  window: InvokingWindow,
  events: Channel<ViewportEvent>,
) -> TauriResult<RenderViewportId> {
  let label: &str = window.get_label();
  let handle: NonZeroIsize = handles
    .get(label)
    .ok_or_else(|| format!("Window '{label}' cannot host a native viewport"))?;
  let host: Arc<dyn RenderWindowHost> = WindowHostBuilder::new(handle).build()?;

  viewports
    .attach(label, host, events)
    .ok_or_else(|| "No native viewport number is left to attach another".to_string())
}
