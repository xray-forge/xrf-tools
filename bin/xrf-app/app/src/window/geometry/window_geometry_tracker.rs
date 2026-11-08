use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;

use tauri::utils::config::WindowConfig;
use tauri::webview::WebviewWindow;
use tauri::{AppHandle, Runtime, WindowEvent};
use xrf_app_core::preferences::{PreferenceKey, Preferences};

use crate::window::geometry::monitor_work_area::MonitorWorkArea;
use crate::window::geometry::window_geometry::WindowGeometry;
use crate::window::geometry::window_geometry_state::WindowGeometryState;

/// Follows the main window so the next run can open where this one left it.
pub(crate) struct WindowGeometryTracker {
  preferences: Preferences,
  state: Mutex<WindowGeometryState>,
}

impl WindowGeometryTracker {
  /// Where the main window was last left.
  const PREFERENCE_KEY: PreferenceKey = PreferenceKey::new("window.geometry");

  /// How long a move or a resize settles before it is written.
  const SETTLE_DELAY: Duration = Duration::from_secs(1);

  /// Where the last run left the window, fitted to the monitors attached now, when they still have room for it.
  pub(crate) fn restore<R: Runtime>(
    application: &AppHandle<R>,
    preferences: &Preferences,
    config: &WindowConfig,
  ) -> Option<WindowGeometry> {
    let saved: WindowGeometry = preferences.read(Self::PREFERENCE_KEY)?;
    let fitted: WindowGeometry = saved.fit(
      &MonitorWorkArea::list(application),
      config.min_width.unwrap_or_default(),
      config.min_height.unwrap_or_default(),
    )?;

    log::info!(
      "Restoring main window geometry: {}x{} at {},{}{}",
      fitted.width,
      fitted.height,
      fitted.x,
      fitted.y,
      if fitted.maximized { ", maximized" } else { "" }
    );

    Some(fitted)
  }

  /// Follow `window`, starting from what it was opened at.
  pub(crate) fn track<R: Runtime>(window: &WebviewWindow<R>, preferences: Preferences, opened: Option<WindowGeometry>) {
    let opened: WindowGeometry = WindowGeometry::measure(window, opened.unwrap_or_default());

    let tracker: Arc<Self> = Arc::new(Self {
      preferences,
      state: Mutex::new(WindowGeometryState::new(opened)),
    });

    let tracked: WebviewWindow<R> = window.clone();

    window.on_window_event(move |event| match event {
      WindowEvent::Moved(_) | WindowEvent::Resized(_) => tracker.record(&tracked),
      WindowEvent::CloseRequested { .. } => tracker.close(&tracked),
      // Nothing is left to measure: Tauri lets go of the native window before it reports it destroyed, and every move
      // and resize up to then was measured as it happened.
      WindowEvent::Destroyed => tracker.save(),
      _ => (),
    });
  }

  /// Take where the window is, and write it once it stops moving.
  fn record<R: Runtime>(self: &Arc<Self>, window: &WebviewWindow<R>) {
    if !self.measure(window).begin_settling() {
      return;
    }

    let tracker: Arc<Self> = Arc::clone(self);

    thread::spawn(move || {
      thread::sleep(Self::SETTLE_DELAY);

      tracker.write();
    });
  }

  /// Take where the window was left and put it on the disk.
  fn close<R: Runtime>(&self, window: &WebviewWindow<R>) {
    drop(self.measure(window));
    self.save();
  }

  /// Put what was taken on the disk now, which no settle delay would outlive the process to do.
  fn save(&self) {
    self.write();

    if self.lock().take_unsaved()
      && let Err(error) = self.preferences.flush()
    {
      log::error!("{error}");
    }
  }

  /// Measure under the lock, and hand it on, because what settles has to be decided about what was just measured.
  fn measure<R: Runtime>(&self, window: &WebviewWindow<R>) -> MutexGuard<'_, WindowGeometryState> {
    let mut state: MutexGuard<'_, WindowGeometryState> = self.lock();
    let measured: WindowGeometry = WindowGeometry::measure(window, state.get_geometry());

    state.record(measured);

    state
  }

  fn write(&self) {
    let geometry: Option<WindowGeometry> = self.lock().take_unwritten();

    if let Some(geometry) = geometry
      && let Err(error) = self.preferences.write(Self::PREFERENCE_KEY, &geometry)
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
