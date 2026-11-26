#![cfg_attr(
  windows,
  expect(
    unsafe_code,
    reason = "loads SDL's library by its path, which the delay-load helper then binds by name"
  )
)]

#[cfg(windows)]
use std::path::{Path, PathBuf};

/// SDL's library, which every binary on Windows delay-loads (`.cargo/config.toml`), so it is found by its full path here
/// rather than on the system's search path.
pub(crate) struct PlatformLibrary;

impl PlatformLibrary {
  /// The library's file name on Windows.
  #[cfg(windows)]
  const FILE: &'static str = "SDL3.dll";

  /// Where `cargo make fetch-sdks` places the library in the checkout, which a development build loads it from.
  #[cfg(windows)]
  const CHECKOUT: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../.sdks/sdl3");

  /// Makes SDL callable: the library beside the executable, as a packaged game ships it, else the checkout's.
  ///
  /// # Errors
  ///
  /// Returns where the library was looked for, or why it does not load.
  #[cfg(windows)]
  pub(crate) fn load() -> Result<(), String> {
    let library: PathBuf = Self::locate().ok_or_else(|| {
      format!(
        "SDL's library {} is neither beside the executable nor in {}: run `cargo make fetch-sdks`",
        Self::FILE,
        Self::CHECKOUT
      )
    })?;

    Self::load_file(&library)?;
    log::info!("SDL loaded from {}", library.display());

    Ok(())
  }

  /// Nothing to load: SDL is linked as the system has it.
  #[cfg(not(windows))]
  pub(crate) fn load() -> Result<(), String> {
    Ok(())
  }

  /// The first place holding the library: beside the executable, then the checkout.
  #[cfg(windows)]
  fn locate() -> Option<PathBuf> {
    let beside: Option<PathBuf> = std::env::current_exe()
      .ok()
      .and_then(|executable| executable.parent().map(|directory| directory.join(Self::FILE)));

    beside
      .into_iter()
      .chain([Path::new(Self::CHECKOUT).join(Self::FILE)])
      .find(|file| file.is_file())
  }

  #[cfg(windows)]
  fn load_file(library: &Path) -> Result<(), String> {
    use windows::Win32::System::LibraryLoader::{LOAD_WITH_ALTERED_SEARCH_PATH, LoadLibraryExW};
    use windows::core::HSTRING;

    // SAFETY: the name is an owned wide string the call reads only during it; the module stays loaded for the process's
    // life, as the delay-load helper then binds every call into it by its name.
    unsafe { LoadLibraryExW(&HSTRING::from(library.as_os_str()), None, LOAD_WITH_ALTERED_SEARCH_PATH) }
      .map(|_| ())
      .map_err(|error| format!("SDL's library {} does not load: {error}", library.display()))
  }
}
