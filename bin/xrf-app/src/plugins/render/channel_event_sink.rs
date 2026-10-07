use tauri::ipc::Channel;
use xrf_renderer::{RenderEventSink, RenderViewportEvent};
use xrf_world::{WorldEventSink, WorldViewportEvent};

use crate::plugins::render::viewport_event::ViewportEvent;

/// A viewport's events, the renderer's and the world's alike, sent down the channel its page attached it with.
pub struct ChannelEventSink {
  channel: Channel<ViewportEvent>,
}

impl ChannelEventSink {
  pub fn new(channel: Channel<ViewportEvent>) -> Self {
    Self { channel }
  }

  fn send(&self, event: ViewportEvent) -> bool {
    // A send to a page that went still succeeds, so the plugin detaches a window's viewports when a page loads over it.
    self.channel.send(event).is_ok()
  }
}

impl RenderEventSink for ChannelEventSink {
  fn send(&self, event: RenderViewportEvent) -> bool {
    ChannelEventSink::send(self, ViewportEvent::Render(event))
  }
}

impl WorldEventSink for ChannelEventSink {
  fn send(&self, event: WorldViewportEvent) -> bool {
    ChannelEventSink::send(self, ViewportEvent::World(event))
  }
}
