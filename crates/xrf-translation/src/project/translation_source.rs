use serde::{Deserialize, Serialize};

/// Where one language's copy of a file was actually found.
///
/// Two paths because they answer different questions. The logical path is the engine identity, which
/// is what the file is; the physical path is where it happens to sit on this machine, which exists
/// only when the winning mount is a loose directory. An archived winner has none, and that absence is
/// the write guard — bytes inside a `.db` volume cannot be edited in place.
///
/// **The physical path is for showing, never for addressing.** It is portable-formatted, so it has
/// already lost any name that is not valid Unicode and any `\` a host treats as an ordinary character.
/// A save renders the edited bytes by logical path (`render_edits`) and commits them through the VFS,
/// which resolves where the file really is.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationSource {
  pub logical_path: String,
  pub physical_path: Option<String>,
}

impl TranslationSource {
  pub fn new(logical_path: impl Into<String>, physical_path: Option<String>) -> Self {
    Self {
      logical_path: logical_path.into(),
      physical_path,
    }
  }

  /// Whether an edit could write this copy back.
  pub fn is_editable(&self) -> bool {
    self.physical_path.is_some()
  }
}
