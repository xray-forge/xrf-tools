#![expect(
  unsafe_code,
  reason = "promises the window host the window's lifetime, which the viewports' close approval keeps"
)]

use std::num::NonZeroIsize;
use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use xrf_app_core::TauriResult;
use xrf_app_core::window::{InvokingWindow, WindowHandles};
use xrf_renderer::{RenderViewportId, RenderWindowHost};
use xrf_renderer_host::WindowHostBuilder;

use crate::viewport::{RenderViewports, ViewportEvent};

/// Start drawing a native viewport into the calling window, telling the page what it costs and where its camera is
/// through `events`.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "attach_viewport")]
pub(crate) fn render_attach_viewport(
  viewports: State<'_, RenderViewports>,
  handles: State<'_, WindowHandles>,
  window: InvokingWindow,
  events: Channel<ViewportEvent>,
) -> TauriResult<RenderViewportId> {
  let label: &str = window.get_label();
  let handle: NonZeroIsize = handles
    .get(label)
    .ok_or_else(|| format!("Window '{label}' cannot host a native viewport"))?;
  // SAFETY: the handle is the one the window registered when built and forgets when destroyed. A window whose close its
  // page approved, while viewports draw into it, is held open until the render thread answers that it handled their
  // detach, and only then destroyed (`RenderViewports::close_window`). Only a page draws viewports, and a page that
  // draws them asks before its window closes, so no close goes ahead unapproved while they draw; nothing in the
  // application destroys a window any other way, and one found destroyed still drawing them is logged as a breach
  // (`RenderViewports::forget_window`).
  let builder: WindowHostBuilder = unsafe { WindowHostBuilder::new(handle) };
  let host: Arc<dyn RenderWindowHost> = builder.build()?;

  viewports.attach(label, host, events)
}
