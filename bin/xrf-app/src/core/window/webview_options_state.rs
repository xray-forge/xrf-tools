use std::sync::Mutex;

use crate::core::types::TauriResult;
use crate::core::window::webview_options::WebviewOptions;

/// Writes a choice where the next start reads it.
pub type TWebviewOptionsStore = Box<dyn Fn(&WebviewOptions) -> TauriResult + Send + Sync>;

/// The webview options this run's main window was built with, and the choice the next start builds it with.
pub struct WebviewOptionsState {
  running: WebviewOptions,
  chosen: Mutex<WebviewOptions>,
  store: TWebviewOptionsStore,
}

impl WebviewOptionsState {
  pub fn new(running: WebviewOptions, store: TWebviewOptionsStore) -> Self {
    Self {
      running,
      chosen: Mutex::new(running),
      store,
    }
  }

  /// What the webview runs with until the application restarts: its browser starts once a run.
  pub fn get_running(&self) -> WebviewOptions {
    self.running
  }

  /// What the next start builds the webview with.
  pub fn get_chosen(&self) -> TauriResult<WebviewOptions> {
    self
      .chosen
      .lock()
      .map(|chosen| *chosen)
      .map_err(|error| format!("Failed to read the chosen webview options: {error}"))
  }

  /// Keep a choice for the next start, stored before it is taken as chosen.
  pub fn choose(&self, options: WebviewOptions) -> TauriResult {
    (self.store)(&options)?;

    *self
      .chosen
      .lock()
      .map_err(|error| format!("Failed to keep the chosen webview options: {error}"))? = options;

    Ok(())
  }
}
