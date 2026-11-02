use tauri::Runtime;
use tauri::webview::WebviewWindow;

/// The animations Windows plays as a window shows, hides, maximizes or minimizes.
///
/// Showing a window plays one too: a maximized window slides in from its restore rectangle, its page drawn up to some
/// 20 px off its place for a quarter of a second, which reads as the page standing above the top of the screen and then
/// jumping into place. A held window is revealed with its page already drawn, so it is shown without one, and the
/// animations are on again once it showed, so the person's own maximize and minimize still play theirs.
pub(crate) struct WindowTransitions;

impl WindowTransitions {
  /// DWM's forced-off flag as the Win32 `BOOL` it reads: the animations are stopped.
  const FORCED_OFF: i32 = 1;

  /// DWM's forced-off flag cleared: the window animates as the system is set to.
  const SYSTEM_SET: i32 = 0;

  /// Stops the animations of `window` until [`Self::enable`].
  pub fn disable<R: Runtime>(window: &WebviewWindow<R>) {
    Self::force_disabled(window, Self::FORCED_OFF);
  }

  /// Lets `window` animate again.
  pub fn enable<R: Runtime>(window: &WebviewWindow<R>) {
    Self::force_disabled(window, Self::SYSTEM_SET);
  }

  /// Sets DWM's forced-off flag for `window` to `value`, one of [`Self::FORCED_OFF`] and [`Self::SYSTEM_SET`].
  #[cfg(windows)]
  fn force_disabled<R: Runtime>(window: &WebviewWindow<R>, value: i32) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{DWMWA_TRANSITIONS_FORCEDISABLED, DwmSetWindowAttribute};

    let hwnd: HWND = match window.hwnd() {
      Ok(hwnd) => hwnd,
      Err(error) => {
        log::warn!(
          "Failed to read the handle of window '{}' to set its animations: {error}",
          window.label()
        );

        return;
      }
    };

    // SAFETY: `hwnd` is the live window's own handle, and the attribute is read as a `BOOL` of the size given.
    let result: windows::core::Result<()> = unsafe {
      DwmSetWindowAttribute(
        hwnd,
        DWMWA_TRANSITIONS_FORCEDISABLED,
        std::ptr::from_ref(&value).cast(),
        size_of::<i32>() as u32,
      )
    };

    if let Err(error) = result {
      log::warn!("Failed to set the animations of window '{}': {error}", window.label());
    }
  }

  /// Nothing, on the platforms whose window managers this does not concern.
  #[cfg(not(windows))]
  fn force_disabled<R: Runtime>(_window: &WebviewWindow<R>, _value: i32) {}
}
