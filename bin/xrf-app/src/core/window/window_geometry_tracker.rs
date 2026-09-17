use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use tauri::webview::WebviewWindow;
use tauri::{PhysicalPosition, PhysicalSize, Runtime, WindowEvent};

use crate::core::preferences::{PreferenceKey, Preferences};
use crate::core::window::window_geometry::WindowGeometry;
use crate::core::window::window_geometry_state::WindowGeometryState;

/// How long a move or a resize settles before it is written.
const SETTLE_DELAY: Duration = Duration::from_secs(1);

/// Follows the main window so the next run can open where this one left it.
pub struct WindowGeometryTracker<R: Runtime> {
  preferences: Preferences<R>,
  state: Mutex<WindowGeometryState>,
}

/// Follow `window`, starting from what it was opened at.
pub fn track_window_geometry<R: Runtime>(
  window: &WebviewWindow<R>,
  preferences: Preferences<R>,
  opened: Option<WindowGeometry>,
) {
  let opened: WindowGeometry = measure_window(window, opened.unwrap_or_default());

  let tracker: Arc<WindowGeometryTracker<R>> = Arc::new(WindowGeometryTracker {
    preferences,
    state: Mutex::new(WindowGeometryState::new(opened)),
  });

  let tracked: WebviewWindow<R> = window.clone();

  window.on_window_event(move |event| match event {
    WindowEvent::Moved(_) | WindowEvent::Resized(_) => tracker.record(&tracked),
    WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed => tracker.close(&tracked),
    _ => (),
  });
}

impl<R: Runtime> WindowGeometryTracker<R> {
  /// Take where the window is, and write it once it stops moving.
  fn record(self: &Arc<Self>, window: &WebviewWindow<R>) {
    let is_settling: bool = match self.measure(window) {
      Some(mut state) => state.begin_settling(),
      None => return,
    };

    if !is_settling {
      return;
    }

    let tracker: Arc<Self> = Arc::clone(self);

    thread::spawn(move || {
      thread::sleep(SETTLE_DELAY);

      tracker.write();
    });
  }

  /// Take where the window was left and put it on the disk, which no settle delay would outlive the process to do.
  fn close(&self, window: &WebviewWindow<R>) {
    self.measure(window);
    self.write();

    if self.lock().is_some_and(|mut state| state.take_unsaved()) {
      self.preferences.flush();
    }
  }

  /// Measure under the lock, and hand it on, because what settles has to be decided about what was just measured.
  fn measure(&self, window: &WebviewWindow<R>) -> Option<MutexGuard<'_, WindowGeometryState>> {
    let mut state: MutexGuard<'_, WindowGeometryState> = self.lock()?;
    let measured: WindowGeometry = measure_window(window, state.get_geometry());

    state.record(measured);

    Some(state)
  }

  fn write(&self) {
    let geometry: Option<WindowGeometry> = self.lock().and_then(|mut state| state.take_unwritten());

    if let Some(geometry) = geometry {
      self.preferences.write(PreferenceKey::WindowGeometry, &geometry);
    }
  }

  fn lock(&self) -> Option<MutexGuard<'_, WindowGeometryState>> {
    match self.state.lock() {
      Ok(state) => Some(state),
      Err(error) => {
        log::error!("Failed to reach the tracked window geometry: {error}");

        None
      }
    }
  }
}

/// Read the window's own rectangle, keeping `previous` for what a window in that state cannot report.
fn measure_window<R: Runtime>(window: &WebviewWindow<R>, previous: WindowGeometry) -> WindowGeometry {
  let is_maximized: bool = read_window_state(window.is_maximized(), "maximized");

  // A maximized or minimized window occupies a rectangle nobody chose; the one it restores down to is already held.
  if is_maximized || read_window_state(window.is_minimized(), "minimized") {
    return WindowGeometry {
      maximized: is_maximized,
      ..previous
    };
  }

  let (position, size, scale) = match measure_rectangle(window) {
    Ok(measured) => measured,
    Err(error) => {
      log::error!("Failed to measure the main window: {error}");

      return previous;
    }
  };

  WindowGeometry {
    x: position.x,
    y: position.y,
    width: size.width,
    height: size.height,
    scale,
    maximized: false,
  }
}

/// The three measurements together, so one unreadable window is one failure rather than three.
fn measure_rectangle<R: Runtime>(
  window: &WebviewWindow<R>,
) -> tauri::Result<(PhysicalPosition<i32>, PhysicalSize<u32>, f64)> {
  Ok((window.outer_position()?, window.inner_size()?, window.scale_factor()?))
}

fn read_window_state(state: tauri::Result<bool>, name: &str) -> bool {
  match state {
    Ok(state) => state,
    Err(error) => {
      log::error!("Failed to read whether the main window is {name}: {error}");

      false
    }
  }
}
