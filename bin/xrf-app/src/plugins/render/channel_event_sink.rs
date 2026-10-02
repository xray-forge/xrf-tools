use tauri::ipc::Channel;
use xrf_renderer::{RenderEventSink, RenderViewportEvent};

/// A viewport's events, sent down the channel its page attached it with.
pub struct ChannelEventSink {
  channel: Channel<RenderViewportEvent>,
}

impl ChannelEventSink {
  pub fn new(channel: Channel<RenderViewportEvent>) -> Self {
    Self { channel }
  }
}

impl RenderEventSink for ChannelEventSink {
  fn send(&self, event: RenderViewportEvent) -> bool {
    self.channel.send(event).is_ok()
  }
}
