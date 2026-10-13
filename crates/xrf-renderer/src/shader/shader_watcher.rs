use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// A debug build's watch over the shader sources, which tells the render thread when a module's file was written, so
/// it reads them again only then rather than looking at every file on a schedule.
pub struct ShaderWatcher {
  /// None in a release build, or where the platform refused a watcher; shaders then never reload.
  watcher: Option<RecommendedWatcher>,
  is_changed: Arc<AtomicBool>,
}

impl ShaderWatcher {
  /// A watcher over the shader sources under `root` in a debug build; one that never sees a change otherwise.
  pub fn new(root: &Path) -> Self {
    let is_changed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    Self {
      watcher: cfg!(debug_assertions)
        .then(|| Self::watch(root, Arc::clone(&is_changed)))
        .flatten(),
      is_changed,
    }
  }

  /// Whether a shader source was written since this was last asked.
  pub fn take_changed(&self) -> bool {
    self.watcher.is_some() && self.is_changed.swap(false, Ordering::AcqRel)
  }

  fn watch(root: &Path, is_changed: Arc<AtomicBool>) -> Option<RecommendedWatcher> {
    let watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
      let Ok(event) = event else {
        return;
      };
      let is_shader: bool = event
        .paths
        .iter()
        .any(|path| path.extension().is_some_and(|extension| extension == "wgsl"));

      if is_shader && matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
        is_changed.store(true, Ordering::Release);
      }
    });

    match watcher.and_then(|mut watcher| watcher.watch(root, RecursiveMode::Recursive).map(|()| watcher)) {
      Ok(watcher) => Some(watcher),
      Err(error) => {
        log::warn!("Shaders will not reload as their files change: {error}");

        None
      }
    }
  }
}
