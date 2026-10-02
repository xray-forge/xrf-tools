use std::sync::mpsc::Sender;

use crate::contract::render_settings::RenderSettings;
use crate::thread::render_command::RenderCommand;

/// What the renderer's handle and its thread share, under one lock: commands are sent under it, and the thread only
/// stops under it after finding none pending, so nothing sent is ever lost to a stopping thread.
#[derive(Default)]
pub struct RenderLink {
  /// The running thread's channel, or `None` while no thread runs.
  pub sender: Option<Sender<RenderCommand>>,
  /// The settings a thread started next begins with.
  pub settings: RenderSettings,
}
