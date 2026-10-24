use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use xrf_renderer::{RenderCommand, RenderQueue, get_texture_name};
use xrf_vfs::XrayRoots;

/// The roots the shown levels and models read, watched for texture files written there, whatever wrote them: this
/// application's editors or another program. Every viewport drawing a texture whose file changed loads it again.
pub(crate) struct TextureWatcher {
  /// None where the platform refused a watcher; textures then reload only on this application's own writes.
  watcher: Option<Mutex<RecommendedWatcher>>,
  watched: Mutex<HashSet<PathBuf>>,
}

impl TextureWatcher {
  /// How long a burst of writes is gathered before the renderer is told, so a file written in parts loads once.
  const SETTLE: Duration = Duration::from_millis(250);

  /// A watcher telling `queue` what changed.
  pub fn new(queue: RenderQueue) -> Self {
    let (sender, receiver): (Sender<String>, Receiver<String>) = channel();
    let watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
      let Ok(event) = event else {
        return;
      };

      if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
        for name in event.paths.iter().filter_map(|path| Self::to_texture_name(path)) {
          let _ = sender.send(name);
        }
      }
    });

    let watcher: Option<Mutex<RecommendedWatcher>> = match watcher {
      Ok(watcher) => std::thread::Builder::new()
        .name("texture watcher".to_owned())
        .spawn(move || Self::forward(&receiver, &queue))
        .inspect_err(|error| log::warn!("Textures will not reload as their files change: {error}"))
        .ok()
        .map(|_| Mutex::new(watcher)),
      Err(error) => {
        log::warn!("Textures will not reload as their files change: {error}");

        None
      }
    };

    Self {
      watcher,
      watched: Mutex::default(),
    }
  }

  /// Watches every root of `roots` not watched yet, and every folder under it.
  pub fn watch(&self, roots: &XrayRoots) {
    let Some(watcher) = &self.watcher else {
      return;
    };
    let mut watched = self.lock_watched();

    for root in &roots.roots {
      if !root.path.is_dir() || watched.contains(&root.path) {
        continue;
      }

      match watcher
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .watch(&root.path, RecursiveMode::Recursive)
      {
        Ok(()) => {
          watched.insert(root.path.clone());
        }
        Err(error) => log::warn!(
          "Textures under '{}' will not reload as their files change: {error}",
          root.path.display()
        ),
      }
    }
  }

  /// Whether a file written at `path` reloads by itself: it lies under a root watched.
  pub fn is_watched(&self, path: &Path) -> bool {
    self.watcher.is_some() && self.lock_watched().iter().any(|root| path.starts_with(root))
  }

  /// A written file's texture name, where it is a texture: a `.dds` file's name, lowercase, without its extension.
  pub fn to_texture_name(path: &Path) -> Option<String> {
    let is_texture: bool = path
      .extension()
      .is_some_and(|extension| extension.eq_ignore_ascii_case("dds"));

    is_texture
      .then(|| path.file_stem())
      .flatten()
      .map(|stem| get_texture_name(&stem.to_string_lossy()))
  }

  /// Tells the renderer the names of the textures written, a settled burst at a time.
  fn forward(receiver: &Receiver<String>, queue: &RenderQueue) {
    while let Ok(first) = receiver.recv() {
      let mut names: HashSet<String> = HashSet::from([first]);

      loop {
        match receiver.recv_timeout(Self::SETTLE) {
          Ok(name) => {
            names.insert(name);
          }
          Err(RecvTimeoutError::Timeout) => break,
          Err(RecvTimeoutError::Disconnected) => return,
        }
      }

      queue.send(RenderCommand::ReloadTextures {
        names: names.into_iter().collect(),
      });
    }
  }

  /// The roots watched; a set changed only whole entry by whole entry, so one a panic left behind is still whole.
  fn lock_watched(&self) -> MutexGuard<'_, HashSet<PathBuf>> {
    self.watched.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
