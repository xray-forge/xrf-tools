/// The name one preference is written under. Each owner declares its own beside the value it keeps, spelled
/// `<owner>.<value>` (`window.geometry`, `audio.settings`), so the store never needs a list of every plugin's.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreferenceKey(&'static str);

impl PreferenceKey {
  pub const fn new(name: &'static str) -> Self {
    Self(name)
  }

  /// The one spelling the file is written with.
  pub const fn as_str(&self) -> &'static str {
    self.0
  }
}
