use std::num::NonZeroIsize;
use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use xrf_app_core::types::TauriResult;
use xrf_app_core::window::WindowHandles;
use xrf_renderer::{RenderViewportId, RenderWindowHost};

use crate::channel_event_sink::ChannelEventSink;
use crate::render_viewports::RenderViewports;
use crate::viewport_event::ViewportEvent;
use crate::window_host::WindowHostBuilder;

/// Start drawing a native viewport into a window, named by its label, telling the page what it costs and where its
/// camera is through `events`.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "attach_viewport"))]
#[tauri::command(rename = "attach_viewport")]
pub fn render_attach_viewport(
  viewports: State<'_, RenderViewports>,
  handles: State<'_, WindowHandles>,
  window: String,
  events: Channel<ViewportEvent>,
) -> TauriResult<RenderViewportId> {
  let handle: NonZeroIsize = handles
    .get(&window)
    .ok_or_else(|| format!("Window '{window}' cannot host a native viewport"))?;
  let host: Arc<dyn RenderWindowHost> = WindowHostBuilder::new(handle).build()?;

  viewports
    .attach(
      &window,
      host,
      (
        Box::new(ChannelEventSink::new(events.clone())),
        Box::new(ChannelEventSink::new(events)),
      ),
    )
    .ok_or_else(|| "No native viewport number is left to attach another".to_string())
}
