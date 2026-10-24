use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;

use tauri::utils::config::WindowConfig;
use tauri::webview::WebviewWindow;
use tauri::{PhysicalPosition, PhysicalSize, Runtime, WindowEvent};

use crate::preferences::{PreferenceKey, Preferences};
use crate::window::geometry::monitor_work_area::MonitorWorkArea;
use crate::window::geometry::window_geometry::WindowGeometry;
use crate::window::geometry::window_geometry_state::WindowGeometryState;

/// Follows the main window so the next run can open where this one left it.
pub struct WindowGeometryTracker<R: Runtime> {
  preferences: Preferences<R>,
  state: Mutex<WindowGeometryState>,
}

impl<R: Runtime> WindowGeometryTracker<R> {
  /// How long a move or a resize settles before it is written.
  const SETTLE_DELAY: Duration = Duration::from_secs(1);

  /// Open the window where the last run left it, when the monitors attached now still have room for it.
  pub fn restore(
    window: &WebviewWindow<R>,
    preferences: &Preferences<R>,
    config: &WindowConfig,
  ) -> Option<WindowGeometry> {
    let saved: WindowGeometry = preferences.read(PreferenceKey::WindowGeometry)?;
    let fitted: WindowGeometry = saved.fit(
      &MonitorWorkArea::list(window),
      config.min_width.unwrap_or_default(),
      config.min_height.unwrap_or_default(),
    )?;

    if let Err(error) = window.set_size(PhysicalSize::new(fitted.width, fitted.height)) {
      log::error!("Failed to open the main window at its saved size: {error}");

      return None;
    }

    if let Err(error) = window.set_position(PhysicalPosition::new(fitted.x, fitted.y)) {
      log::error!("Failed to open the main window at its saved position: {error}");

      return None;
    }

    // After the rectangle rather than before it: this is what the window restores down to.
    if fitted.maximized
      && let Err(error) = window.maximize()
    {
      log::error!("Failed to open the main window maximized: {error}");
    }

    log::info!(
      "Restored main window geometry: {}x{} at {},{}{}",
      fitted.width,
      fitted.height,
      fitted.x,
      fitted.y,
      if fitted.maximized { ", maximized" } else { "" }
    );

    Some(fitted)
  }

  /// Follow `window`, starting from what it was opened at.
  pub fn track(window: &WebviewWindow<R>, preferences: Preferences<R>, opened: Option<WindowGeometry>) {
    let opened: WindowGeometry = WindowGeometry::measure(window, opened.unwrap_or_default());

    let tracker: Arc<Self> = Arc::new(Self {
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

  /// Take where the window is, and write it once it stops moving.
  fn record(self: &Arc<Self>, window: &WebviewWindow<R>) {
    if !self.measure(window).begin_settling() {
      return;
    }

    let tracker: Arc<Self> = Arc::clone(self);

    thread::spawn(move || {
      thread::sleep(Self::SETTLE_DELAY);

      tracker.write();
    });
  }

  /// Take where the window was left and put it on the disk, which no settle delay would outlive the process to do.
  fn close(&self, window: &WebviewWindow<R>) {
    drop(self.measure(window));
    self.write();

    if self.lock().take_unsaved()
      && let Err(error) = self.preferences.flush()
    {
      log::error!("{error}");
    }
  }

  /// Measure under the lock, and hand it on, because what settles has to be decided about what was just measured.
  fn measure(&self, window: &WebviewWindow<R>) -> MutexGuard<'_, WindowGeometryState> {
    let mut state: MutexGuard<'_, WindowGeometryState> = self.lock();
    let measured: WindowGeometry = WindowGeometry::measure(window, state.get_geometry());

    state.record(measured);

    state
  }

  fn write(&self) {
    let geometry: Option<WindowGeometry> = self.lock().take_unwritten();

    if let Some(geometry) = geometry
      && let Err(error) = self.preferences.write(PreferenceKey::WindowGeometry, &geometry)
    {
      log::error!("{error}");
    }
  }

  /// The tracked state, recovered from a poisoned lock: every update replaces whole fields, so a panic mid-update
  /// leaves a state that is merely one record behind.
  fn lock(&self) -> MutexGuard<'_, WindowGeometryState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
