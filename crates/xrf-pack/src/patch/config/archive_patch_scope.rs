use xrf_error::{XrfError, XrfResult};
use xrf_vfs::XrayLogicalPath;

/// What a comparison is allowed to look at, in engine terms and decided once.
#[derive(Debug, Default)]
pub(crate) struct ArchivePatchScope {
  include: Vec<String>,
  ignore: Vec<String>,
  /// Suffixes with their dot, lower-cased, as `*.txt` reduces to `.txt`.
  extensions: Vec<String>,
}

impl ArchivePatchScope {
  /// Fold the authored prefixes and extension patterns into the forms the predicate compares.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for a prefix that is not a name the engine can address.
  pub(crate) fn new(include: &[String], ignore: &[String], extensions: &[String]) -> XrfResult<Self> {
    Ok(Self {
      include: Self::fold_prefixes(include)?,
      ignore: Self::fold_prefixes(ignore)?,
      extensions: extensions
        .iter()
        .map(|pattern| pattern.trim_start_matches('*').to_ascii_lowercase())
        .collect(),
    })
  }

  /// Whether one engine identity is in scope.
  ///
  /// Include, then ignore, then extensions. An empty include list means the whole world, and an ignore always beats an
  /// include, so the narrowing rule can be read without knowing what the other list holds.
  pub(crate) fn admits(&self, name: &str) -> bool {
    if !self.include.is_empty() && !self.matches_any(&self.include, name) {
      return false;
    }

    if self.matches_any(&self.ignore, name) {
      return false;
    }

    !self.extensions.iter().any(|suffix| name.ends_with(suffix.as_str()))
  }

  /// Whether the scope names a subtree, which is what tells an empty answer from a selection matching nothing.
  pub(crate) fn is_narrowed(&self) -> bool {
    !self.include.is_empty()
  }

  fn matches_any(&self, prefixes: &[String], name: &str) -> bool {
    prefixes
      .iter()
      .any(|prefix| XrayLogicalPath::is_component_prefix(name, prefix))
  }

  fn fold_prefixes(prefixes: &[String]) -> XrfResult<Vec<String>> {
    prefixes
      .iter()
      .map(|prefix| {
        XrayLogicalPath::normalize(prefix).map_err(|error| {
          XrfError::new_invalid_error(format!(
            "Patch scope '{prefix}' is not a name the engine can address: {error}"
          ))
        })
      })
      .collect()
  }
}
