use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::contract::render_settings::RenderSettings;
use crate::host::render_bundle::RenderBundle;
use crate::host::render_world::RenderWorld;
use crate::thread::render_command::RenderCommand;
use crate::thread::render_link::RenderLink;
use crate::thread::render_thread::RenderThread;
use crate::thread::render_workers::RenderWorkers;

/// The way to the render thread: cheap to make, and touching no GPU until a viewport is attached.
///
/// Every command is a message to the thread, started by the first attach and stopped a few seconds after the last
/// detach, so a tool that never draws never starts a GPU.
pub struct RenderQueue {
  link: Arc<Mutex<RenderLink>>,
  /// The pool every thread it starts reads and decodes on.
  workers: RenderWorkers,
  /// The files the renderer ships with, which the application reads for it.
  bundle: Arc<dyn RenderBundle>,
  /// The world it draws, run on its thread before each frame.
  world: Arc<Mutex<dyn RenderWorld>>,
}

impl RenderQueue {
  pub fn new(workers: RenderWorkers, bundle: Arc<dyn RenderBundle>, world: Arc<Mutex<dyn RenderWorld>>) -> Self {
    Self {
      link: Arc::default(),
      workers,
      bundle,
      world,
    }
  }

  /// Hands a command to the running thread, starting one for an attach; any other command has nothing to act on
  /// without a thread, since a stopped thread had no viewport left. Settings are kept for every thread started later.
  pub fn send(&self, command: RenderCommand) {
    let mut link: MutexGuard<'_, RenderLink> = self.lock();
    let is_attach: bool = matches!(command, RenderCommand::Attach { .. });

    if let RenderCommand::Settings { settings } = &command {
      link.settings = *settings;
    }

    let command: RenderCommand = match &link.sender {
      Some(sender) => match sender.send(command) {
        Ok(()) => return,
        // The thread died: its viewports with it.
        Err(error) => error.0,
      },
      None => command,
    };

    link.sender = None;

    if is_attach {
      // A thread that cannot start leaves the viewport undrawn; the next attach tries again.
      let Some(sender) = self.spawn(&mut link) else {
        return;
      };

      if sender.send(command).is_err() {
        log::error!("The render thread stopped before its first viewport arrived");
      }
    }
  }

  fn spawn(&self, link: &mut RenderLink) -> Option<Sender<RenderCommand>> {
    let (sender, receiver) = channel();
    let shared: Arc<Mutex<RenderLink>> = Arc::clone(&self.link);
    let settings: RenderSettings = link.settings;
    let workers: RenderWorkers = self.workers.clone();
    let bundle: Arc<dyn RenderBundle> = Arc::clone(&self.bundle);
    let world: Arc<Mutex<dyn RenderWorld>> = Arc::clone(&self.world);

    if let Err(error) = std::thread::Builder::new()
      .name("xrf-render".into())
      .spawn(move || RenderThread::new(receiver, shared, settings, workers, (bundle, world)).run())
    {
      log::error!("The render thread could not be started: {error}");

      return None;
    }

    link.sender = Some(sender.clone());

    Some(sender)
  }

  fn lock(&self) -> MutexGuard<'_, RenderLink> {
    self.link.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
  }
}
