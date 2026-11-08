#![cfg_attr(
  windows,
  expect(
    unsafe_code,
    reason = "loads Media Foundation's library through Win32, which has no safe binding"
  )
)]

/// Libraries a recording's video is written through, linked lazily: on Windows Media Foundation's are delay-loaded, so
/// loading each from System32 first is what every later call binds to, and a Windows without them (an N edition
/// lacking the Media Feature Pack) answers why rather than failing on its first call. Elsewhere nothing writes video.
#[derive(Clone, Copy, Debug)]
pub struct MediaProbe {
  libraries: &'static [&'static str],
}

impl MediaProbe {
  /// Media Foundation's: its platform, the Sink Writer and the media sinks.
  pub const MEDIA_FOUNDATION: Self = Self::new(&["mfplat.dll", "mfreadwrite.dll", "mf.dll"]);

  /// A probe of `libraries`, loaded from System32.
  pub const fn new(libraries: &'static [&'static str]) -> Self {
    Self { libraries }
  }

  /// Makes the libraries callable, answering why video cannot be written where one does not load.
  ///
  /// # Errors
  ///
  /// Returns an error naming the first library that does not load, or, off Windows, that only Windows writes video.
  pub fn load(&self) -> Result<(), String> {
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

    // SAFETY: the name is a terminated wide string, and the module stays loaded for the process's life, as the
    // delay-load helper then binds every call to it by its name.
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
