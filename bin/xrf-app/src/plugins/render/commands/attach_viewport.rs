use std::num::NonZeroIsize;
use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use xrf_renderer::{RenderViewportId, RenderWindowHost};

use crate::core::types::TauriResult;
use crate::core::window::WindowHandles;
use crate::plugins::render::channel_event_sink::ChannelEventSink;
use crate::plugins::render::state::RenderState;
use crate::plugins::render::viewport_event::ViewportEvent;
use crate::plugins::render::window_host::WindowHostBuilder;

/// Start drawing a native viewport into a window, named by its label, telling the page what it costs and where its
/// camera is through `events`.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "attach_viewport"))]
#[tauri::command(rename = "attach_viewport")]
pub fn render_attach_viewport(
  state: State<'_, RenderState>,
  handles: State<'_, WindowHandles>,
  window: String,
  events: Channel<ViewportEvent>,
) -> TauriResult<RenderViewportId> {
  let handle: NonZeroIsize = handles
    .get(&window)
    .ok_or_else(|| format!("Window '{window}' cannot host a native viewport"))?;
  let host: Arc<dyn RenderWindowHost> = WindowHostBuilder::new(handle).build()?;

  state
    .attach_viewport(
      &window,
      host,
      (
        Box::new(ChannelEventSink::new(events.clone())),
        Box::new(ChannelEventSink::new(events)),
      ),
    )
    .ok_or_else(|| "No native viewport number is left to attach another".to_string())
}
