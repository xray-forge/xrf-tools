use std::process::ExitCode;

/// The application failing to start, before any window is up.
pub struct ApplicationFailure;

impl ApplicationFailure {
  /// Logs why the application cannot start and tells the user, whom a release build on Windows has no console to tell,
  /// only a log file they do not know to open; answers the failure the process ends with.
  pub fn report(message: &str) -> ExitCode {
    log::error!("{message}");

    #[cfg(all(windows, not(debug_assertions)))]
    {
      use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
      use windows::core::HSTRING;

      // SAFETY: both strings outlive the call, and it names no owner window.
      unsafe {
        MessageBoxW(
          None,
          &HSTRING::from(message),
          &HSTRING::from("The application cannot start"),
          MB_OK | MB_ICONERROR,
        )
      };
    }

    ExitCode::FAILURE
  }
}
