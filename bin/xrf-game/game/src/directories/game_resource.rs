use std::path::PathBuf;

/// A file or folder the game ships beside its executable, which a development build finds in the checkout instead,
/// where `cargo make fetch-sdks` or a crate's sources put it.
pub(crate) struct GameResource {
  /// Its path beside the executable, as a packaged game places it.
  pub packaged: &'static str,
  /// Its absolute path in the checkout.
  pub checkout: &'static str,
}

impl GameResource {
  /// The first place holding it: beside the executable, then the checkout; none where neither does.
  pub(crate) fn locate(&self) -> Option<PathBuf> {
    let beside: Option<PathBuf> = std::env::current_exe()
      .ok()
      .and_then(|executable| executable.parent().map(|directory| directory.join(self.packaged)));

    beside
      .into_iter()
      .chain([PathBuf::from(self.checkout)])
      .find(|path| path.exists())
  }
}
