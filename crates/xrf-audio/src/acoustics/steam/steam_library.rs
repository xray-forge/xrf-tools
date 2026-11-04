use std::path::{Path, PathBuf};

use crate::acoustics::audio_acoustics::AudioAcoustics;

/// Steam Audio's library, linked lazily: on Windows it is delay-loaded, so loading it from its directory first is what
/// every later call binds to; on macOS it is weakly linked, found at start from the executable's search paths, and only
/// checked here.
pub(crate) struct SteamLibrary;

impl SteamLibrary {
  /// Makes the library in `directory` callable, answering why it cannot be.
  pub(crate) fn load(directory: &Path) -> Result<(), String> {
    let library: PathBuf = directory.join(AudioAcoustics::LIBRARY);

    if !library.is_file() {
      return Err(format!("Steam Audio's library is not at {}", library.display()));
    }

    Self::load_platform(&library)
  }

  #[cfg(windows)]
  fn load_platform(library: &Path) -> Result<(), String> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    unsafe extern "system" {
      fn LoadLibraryExW(name: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
      fn GetLastError() -> u32;
    }

    /// Resolves the library's own dependencies from its directory.
    const LOAD_WITH_ALTERED_SEARCH_PATH: u32 = 0x8;

    let wide: Vec<u16> = library.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: the name is a terminated wide string, and the module stays loaded for the process's life, as the
    // delay-load helper then binds every call to it by its name.
    let module: *mut c_void =
      unsafe { LoadLibraryExW(wide.as_ptr(), std::ptr::null_mut(), LOAD_WITH_ALTERED_SEARCH_PATH) };

    if module.is_null() {
      // SAFETY: reads the calling thread's last error, which the failed load just set.
      let error: u32 = unsafe { GetLastError() };

      return Err(format!("{} does not load (error {error})", library.display()));
    }

    Ok(())
  }

  #[cfg(not(windows))]
  fn load_platform(library: &Path) -> Result<(), String> {
    use std::ffi::{c_char, c_void};

    unsafe extern "C" {
      fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }

    /// Searches every image loaded at start: `RTLD_DEFAULT`.
    #[cfg(target_os = "macos")]
    const DEFAULT: *mut c_void = -2isize as *mut c_void;
    #[cfg(not(target_os = "macos"))]
    const DEFAULT: *mut c_void = std::ptr::null_mut();

    // SAFETY: looks a terminated name up among the loaded images; nothing is called through it.
    let symbol: *mut c_void = unsafe { dlsym(DEFAULT, c"iplContextCreate".as_ptr()) };

    if symbol.is_null() {
      return Err(format!(
        "Steam Audio was not loaded with the application, though {} is there",
        library.display()
      ));
    }

    Ok(())
  }
}
