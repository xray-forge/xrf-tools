#![cfg_attr(
  windows,
  expect(
    unsafe_code,
    reason = "loads Media Foundation's library through Win32, which has no safe binding"
  )
)]

/// The delay-loaded libraries video is written through. Loading each from System32 first fixes what later calls bind
/// to, and lets a Windows without them (an N edition lacking the Media Feature Pack) say why instead of faulting on the
/// first call. Nothing writes video off Windows.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MediaProbe {
  libraries: &'static [&'static str],
}

impl MediaProbe {
  /// Media Foundation's platform, Sink Writer and media sinks.
  pub(crate) const MEDIA_FOUNDATION: Self = Self::new(&["mfplat.dll", "mfreadwrite.dll", "mf.dll"]);

  /// A probe of `libraries`, loaded from System32.
  pub(crate) const fn new(libraries: &'static [&'static str]) -> Self {
    Self { libraries }
  }

  /// Load the libraries so their calls bind.
  ///
  /// # Errors
  ///
  /// Returns an error naming the first library that does not load, or off Windows, that only Windows writes video.
  pub(crate) fn load(&self) -> Result<(), String> {
    self
      .libraries
      .iter()
      .try_for_each(|library| Self::load_platform(library))
  }

  #[cfg(windows)]
  fn load_platform(library: &str) -> Result<(), String> {
    use windows::Win32::System::LibraryLoader::{LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW};
    use windows::core::{HSTRING, PCWSTR};

    let name: HSTRING = HSTRING::from(library);

    // SAFETY: the name is a terminated wide string; the module stays loaded for the process's life, so the
    // delay-load helper binds every later call to it.
    match unsafe { LoadLibraryExW(PCWSTR(name.as_ptr()), None, LOAD_LIBRARY_SEARCH_SYSTEM32) } {
      Ok(_) => Ok(()),
      Err(error) => Err(format!(
        "{library} does not load ({}): this Windows has no Media Foundation, as an N edition without the Media Feature \
         Pack has none",
        error.message()
      )),
    }
  }

  #[cfg(not(windows))]
  fn load_platform(_library: &str) -> Result<(), String> {
    Err(String::from(
      "Video is written through Media Foundation, which only Windows has",
    ))
  }
}
