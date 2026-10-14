use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::context::library_versions::describe_libraries;
use crate::contract::render_libraries::RenderLibraries;
use crate::contract::render_settings::RenderSettings;
use crate::host::render_bundle::RenderBundle;
use crate::host::render_cache_store::RenderCacheStore;
use crate::host::render_world::RenderWorld;
use crate::thread::render_command::RenderCommand;
use crate::thread::render_link::RenderLink;
use crate::thread::render_thread::RenderThread;
use crate::thread::render_workers::RenderWorkers;

/// The way to the render thread: cheap to make, and touching no GPU until a viewport is attached.
///
/// Every command is a message to the thread, started by the first attach and stopped a few seconds after the last
/// detach, so a tool that never draws never starts a GPU. A clone reaches the same thread.
#[derive(Clone)]
pub struct RenderQueue {
  link: Arc<Mutex<RenderLink>>,
  /// The pool every thread it starts reads and decodes on.
  workers: RenderWorkers,
  /// The files the renderer ships with, which the application reads for it.
  bundle: Arc<dyn RenderBundle>,
  /// The world it draws, run on its thread before each frame.
  world: Arc<Mutex<dyn RenderWorld>>,
  /// Where the driver's compiled pipelines are kept between runs, none where nothing keeps them.
  store: Option<Arc<dyn RenderCacheStore>>,
}

impl RenderQueue {
  pub fn new(workers: RenderWorkers, bundle: Arc<dyn RenderBundle>, world: Arc<Mutex<dyn RenderWorld>>) -> Self {
    Self {
      link: Arc::default(),
      workers,
      bundle,
      world,
      store: None,
    }
  }

  /// The same, keeping the driver's compiled pipelines in `store` between runs.
  pub fn with_cache_store(mut self, store: Arc<dyn RenderCacheStore>) -> Self {
    self.store = Some(store);
    self
  }

  /// The graphics libraries the renderer ships, the DLSS runtime looked for in its bundle.
  pub fn describe_libraries(&self) -> RenderLibraries {
    describe_libraries(self.bundle.as_ref())
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

  /// Waits up to `wait` until the running thread has handled every command sent before; at once where none runs.
  /// Answers whether it did in time.
  pub fn wait_handled(&self, wait: Duration) -> bool {
    let (reply, answer): (Sender<()>, Receiver<()>) = channel();

    self.send(RenderCommand::Handled { reply });

    Self::await_reply(&answer, wait)
  }

  /// Detaches every viewport and stops the running thread at once, waiting up to `wait` until it let the GPU go; at
  /// once where none runs. Answers whether it did in time.
  pub fn stop(&self, wait: Duration) -> bool {
    let (reply, answer): (Sender<()>, Receiver<()>) = channel();

    self.send(RenderCommand::Stop { reply });

    Self::await_reply(&answer, wait)
  }

  /// Whether a reply came within `wait`, or its sender went unanswered: dropped with no thread to take the command.
  fn await_reply(answer: &Receiver<()>, wait: Duration) -> bool {
    !matches!(answer.recv_timeout(wait), Err(RecvTimeoutError::Timeout))
  }

  fn spawn(&self, link: &mut RenderLink) -> Option<Sender<RenderCommand>> {
    let (sender, receiver) = channel();
    let shared: Arc<Mutex<RenderLink>> = Arc::clone(&self.link);
    let settings: RenderSettings = link.settings;
    let workers: RenderWorkers = self.workers.clone();
    let bundle: Arc<dyn RenderBundle> = Arc::clone(&self.bundle);
    let world: Arc<Mutex<dyn RenderWorld>> = Arc::clone(&self.world);
    let store: Option<Arc<dyn RenderCacheStore>> = self.store.clone();
    let previous: Option<JoinHandle<()>> = link.thread.take();
    let started: std::io::Result<JoinHandle<()>> =
      std::thread::Builder::new().name("xrf-render".into()).spawn(move || {
        // The thread before stopped taking commands, but may still be letting its GPU go: a device and a window's
        // swapchain are started only once it has. Commands sent meanwhile wait in this thread's channel, so no sender
        // waits for it.
        if let Some(previous) = previous {
          let _ = previous.join();
        }

        RenderThread::new(receiver, shared, settings, workers, (bundle, world, store)).run();
      });

    match started {
      Ok(thread) => link.thread = Some(thread),
      Err(error) => {
        log::error!("The render thread could not be started: {error}");

        return None;
      }
    }

    link.sender = Some(sender.clone());

    Some(sender)
  }

  fn lock(&self) -> MutexGuard<'_, RenderLink> {
    self.link.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
  }
}
