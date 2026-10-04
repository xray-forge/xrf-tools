use tauri::State;
use tauri::ipc::Channel;
use xrf_renderer::{RenderViewportEvent, RenderViewportId};

use crate::core::types::TauriResult;
use crate::core::window::WindowHandles;
use crate::plugins::render::state::RenderState;

/// Start drawing a native viewport into a window, named by its label, telling the page what it costs and where its
/// camera is through `events`.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "attach_viewport"))]
#[tauri::command(rename = "attach_viewport")]
pub fn render_attach_viewport(
  state: State<'_, RenderState>,
  handles: State<'_, WindowHandles>,
  window: String,
  events: Channel<RenderViewportEvent>,
) -> TauriResult<RenderViewportId> {
  attach(&state, &handles, &window, events)
}

#[cfg(windows)]
fn attach(
  state: &RenderState,
  handles: &WindowHandles,
  window: &str,
  events: Channel<RenderViewportEvent>,
) -> TauriResult<RenderViewportId> {
  use std::sync::Arc;

  use crate::plugins::render::channel_event_sink::ChannelEventSink;
  use crate::plugins::render::win32_window_host::Win32WindowHost;

  let handle = handles
    .get(window)
    .ok_or_else(|| format!("Window '{window}' cannot host a native viewport"))?;
  Ok(state.attach_viewport(
    window,
    Arc::new(Win32WindowHost::new(handle)),
    Box::new(ChannelEventSink::new(events)),
  ))
}

#[cfg(not(windows))]
fn attach(
  _state: &RenderState,
  _handles: &WindowHandles,
  _window: &str,
  _events: Channel<RenderViewportEvent>,
) -> TauriResult<RenderViewportId> {
  // todo: Host native viewports outside Windows, where the webview's window composes differently.
  Err("Native viewports are drawn on Windows only".to_string())
}
