use tauri::async_runtime::{Receiver, Sender, channel};
use tauri::{AppHandle, Runtime};

use crate::core::types::TauriResult;
use crate::plugins::system::diagnostics::memory::WebviewProcess;

type WebviewProcessListing = TauriResult<Vec<WebviewProcess>>;

type WebviewProcessDispatch = dyn Fn(Sender<WebviewProcessListing>) -> TauriResult + Send + Sync;

/// Asks the main window's webview which processes it runs in, on the thread its environment belongs to.
///
/// Holds the application behind a closure so the state is not generic over the runtime the commands cannot name.
pub struct WebviewProcessProbe {
  dispatch: Box<WebviewProcessDispatch>,
}

impl WebviewProcessProbe {
  pub fn new<R: Runtime>(application: AppHandle<R>) -> Self {
    // The window `tauri.conf.json` declares first is the one `build_main_window` builds.
    let label: Option<String> = application
      .config()
      .app
      .windows
      .first()
      .map(|window| window.label.clone());

    Self {
      dispatch: Box::new(move |sender| dispatch_listing(&application, label.as_deref(), sender)),
    }
  }

  /// List the webview's processes, waiting for the webview's thread to answer.
  pub async fn list(&self) -> WebviewProcessListing {
    let (sender, mut receiver): (Sender<WebviewProcessListing>, Receiver<WebviewProcessListing>) = channel(1);

    (self.dispatch)(sender)?;

    receiver
      .recv()
      .await
      .ok_or_else(|| String::from("The webview closed before it listed its processes"))?
  }
}

#[cfg(windows)]
fn dispatch_listing<R: Runtime>(
  application: &AppHandle<R>,
  label: Option<&str>,
  sender: Sender<WebviewProcessListing>,
) -> TauriResult {
  use tauri::Manager;
  use tauri::webview::WebviewWindow;

  let window: WebviewWindow<R> = label
    .and_then(|label| application.get_webview_window(label))
    .ok_or("The main window is not open")?;

  window
    .with_webview(move |platform| {
      // The receiver waits for exactly this one answer, so a full or closed channel has nobody left to tell.
      let _ = sender.try_send(list_environment_processes(&platform));
    })
    .map_err(|error| format!("Failed to reach the main window's webview: {error}"))
}

/// List nothing, on the platforms whose webview is not WebView2.
#[cfg(not(windows))]
fn dispatch_listing<R: Runtime>(
  _application: &AppHandle<R>,
  _label: Option<&str>,
  sender: Sender<WebviewProcessListing>,
) -> TauriResult {
  sender
    .try_send(Ok(Vec::new()))
    .map_err(|error| format!("Failed to answer the webview process listing: {error}"))
}

/// Every process of the webview's environment: browser, renderers, GPU and utilities.
#[cfg(windows)]
fn list_environment_processes(platform: &tauri::webview::PlatformWebview) -> WebviewProcessListing {
  use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_PROCESS_KIND, ICoreWebView2Environment8, ICoreWebView2ProcessInfo, ICoreWebView2ProcessInfoCollection,
  };
  use windows::core::Interface;

  let read = || -> windows::core::Result<Vec<WebviewProcess>> {
    let environment: ICoreWebView2Environment8 = platform.environment().cast()?;
    let mut count: u32 = 0;

    // SAFETY: `with_webview` runs this on the thread the environment was created on, which WebView2 requires.
    let collection: ICoreWebView2ProcessInfoCollection = unsafe { environment.GetProcessInfos()? };

    // SAFETY: as above; `count` outlives the call.
    unsafe { collection.Count(&mut count)? };

    let mut processes: Vec<WebviewProcess> = Vec::with_capacity(count as usize);

    for index in 0..count {
      let mut pid: i32 = 0;
      let mut kind: COREWEBVIEW2_PROCESS_KIND = COREWEBVIEW2_PROCESS_KIND::default();

      // SAFETY: as above; `index` is below the collection's own count, and both outputs outlive the calls.
      let info: ICoreWebView2ProcessInfo = unsafe { collection.GetValueAtIndex(index)? };
      unsafe {
        info.ProcessId(&mut pid)?;
        info.Kind(&mut kind)?;
      }

      if let Ok(pid) = u32::try_from(pid) {
        processes.push(WebviewProcess { kind: kind.into(), pid });
      }
    }

    Ok(processes)
  };

  read().map_err(|error| format!("Failed to list the webview's processes: {error}"))
}
