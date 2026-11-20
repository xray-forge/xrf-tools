#![expect(
  unsafe_code,
  reason = "starts and stops Media Foundation, whose COM calls the `windows` crate exposes as unsafe"
)]

use windows::Win32::Media::MediaFoundation::{MF_VERSION, MFSTARTUP_LITE, MFShutdown, MFStartup};
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};

/// COM and Media Foundation started on the calling thread, held by a recording thread for its life and shut down on
/// drop. Release every Media Foundation object the thread made before dropping it.
pub(crate) struct MediaSession;

impl MediaSession {
  /// Start both on this thread; `MediaProbe` must have loaded the libraries.
  ///
  /// # Errors
  ///
  /// Returns an error when COM or Media Foundation does not start.
  pub(crate) fn start() -> Result<Self, String> {
    // SAFETY: the recording's own thread joins the multithreaded apartment once, leaving it on drop or below.
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
      .ok()
      .map_err(|error| format!("COM does not start for the recording: {}", error.message()))?;

    // SAFETY: the libraries are loaded; the session's drop shuts Media Foundation down once. Lite: a recording writes
    // a file, with no network sources.
    if let Err(error) = unsafe { MFStartup(MF_VERSION, MFSTARTUP_LITE) } {
      // SAFETY: leaves the apartment joined above, on the same thread.
      unsafe { CoUninitialize() };

      return Err(format!("Media Foundation does not start: {}", error.message()));
    }

    Ok(Self)
  }
}

impl Drop for MediaSession {
  fn drop(&mut self) {
    // SAFETY: once each, on the thread that started them, after the thread released what it made.
    unsafe {
      let _ = MFShutdown();
      CoUninitialize();
    }
  }
}
