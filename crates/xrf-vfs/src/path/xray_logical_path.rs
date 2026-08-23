use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::path::{Path, PathBuf};

use serde::Serialize;
use xrf_error::{XrfError, XrfResult};

/// An X-Ray logical path: lower case, backslash separated, with no empty, `.` or `..` component.
///
/// Being separator-explicit is what makes it portable: it splits on `\` itself rather than deferring to
/// `std::path`, so `parent` and `file_name` answer the same on Linux as on Windows, where a `std::path::Path`
/// would treat the whole thing as one component.
///
/// Serialized and typed transparently as its string form, so an engine path crosses IPC as the text the engine uses.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
#[cfg_attr(feature = "typescript-bindings", specta(transparent))]
pub struct XrayLogicalPath(String);

impl XrayLogicalPath {
  /// Normalizes a reference into an engine identity.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is empty or holds an empty, `.` or `..` component.
  pub fn new(path: &str) -> XrfResult<Self> {
    Ok(Self(normalize(path)?.into_owned()))
  }

  /// Wraps a string [`normalize`] already produced.
  ///
  /// Crate-internal, because only the mount layer knows a path came out of normalization. It exists so enumerating tens of
  /// thousands of entries does not re-validate each one to say what the source already guaranteed.
  pub(crate) fn from_normalized(path: String) -> Self {
    debug_assert_eq!(
      normalize(&path).ok().as_deref(),
      Some(path.as_str()),
      "from_normalized was handed a path normalization would have changed"
    );

    Self(path)
  }

  /// The normalized path, for lookups and messages.
  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// Appends a component below this path.
  ///
  /// # Errors
  ///
  /// Returns an error when the result is not a valid logical path.
  pub fn join(&self, component: &str) -> XrfResult<Self> {
    Ok(Self(join(&self.0, component)?))
  }

  /// The directory holding this path, or `None` when it names a top-level entry.
  pub fn parent(&self) -> Option<Self> {
    self.0.rsplit_once('\\').map(|(parent, _)| Self(parent.to_string()))
  }

  /// The final component, which for a file is its name and extension.
  pub fn file_name(&self) -> &str {
    self.0.rsplit_once('\\').map_or(self.0.as_str(), |(_, name)| name)
  }

  /// Whether the path carries `extension`, which is compared without case.
  ///
  /// `extension` is matched with its leading dot, as in `.ltx`.
  pub fn has_extension(&self, extension: &str) -> bool {
    has_extension(&self.0, extension)
  }

  /// Whether this path sits under `prefix`, matching on component boundaries.
  ///
  /// # Errors
  ///
  /// Returns an error when `prefix` is not a valid logical path.
  pub fn is_under(&self, prefix: &str) -> XrfResult<bool> {
    Ok(is_component_prefix(&self.0, &normalize(prefix)?))
  }

  /// Normalizes a path into the canonical X-Ray logical form: lower case, backslash separated, no leading or trailing
  /// separator.
  ///
  /// Exposed because an out-of-crate [`crate::XrayAssetSource`] cannot key its entries correctly without the same rule.
  /// Paths handed to a source are already normalized; a source normalizes only its own keys. A source keys a map of many
  /// thousands of names, which is why this answers a `String` rather than an [`XrayLogicalPath`].
  ///
  /// # Errors
  ///
  /// Returns an error when the path contains an empty, `.` or `..` component.
  pub fn normalize(path: &str) -> XrfResult<String> {
    normalize_logical(path)
  }

  /// Whether a logical path sits under a prefix, matching on component boundaries so `configs_backup` does not match
  /// `configs`.
  ///
  /// Exposed for the same reason as [`Self::normalize`]: an out-of-crate source must scope its enumeration by the same
  /// rule, and two copies that drift would make scoping depend on which kind of source answered. Both arguments are
  /// expected to be normalized already.
  pub fn is_component_prefix(path: &str, prefix: &str) -> bool {
    is_component_prefix(path, prefix)
  }
}

impl Display for XrayLogicalPath {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    formatter.write_str(&self.0)
  }
}

/// Normalizes a path into the canonical X-Ray logical form, behind [`XrayLogicalPath::normalize`].
///
/// # Errors
///
/// Returns an error when the path contains an empty, `.` or `..` component.
pub(crate) fn normalize_logical(path: &str) -> XrfResult<String> {
  Ok(normalize(path)?.into_owned())
}

/// Whether a logical path sits under a prefix, matching on component boundaries so `configs_backup` does not match
/// `configs`. Behind [`XrayLogicalPath::is_component_prefix`]. Both arguments are expected to be normalized already.
pub(crate) fn is_component_prefix(path: &str, prefix: &str) -> bool {
  path == prefix || path.strip_prefix(prefix).is_some_and(|rest| rest.starts_with('\\'))
}

/// Normalizes a path, borrowing it when it is already canonical.
///
/// Rewriting cost three allocations and two full passes to produce an identical string, and enumeration calls this once
/// per entry — tens of thousands of times per run, on paths a source already keyed canonically. Checking first is a
/// single pass with no allocation, so the common case now copies nothing.
pub(crate) fn normalize(path: &str) -> XrfResult<Cow<'_, str>> {
  if is_canonical(path) {
    validate_components(path, path)?;

    return Ok(Cow::Borrowed(path));
  }

  let rewritten: String = path.replace('/', "\\").to_lowercase();
  let trimmed: &str = rewritten.trim_matches('\\');

  validate_components(trimmed, path)?;

  Ok(Cow::Owned(trimmed.to_string()))
}

/// Whether a path is already in the form [`normalize`] would produce.
fn is_canonical(path: &str) -> bool {
  !path.is_empty()
    && !path.starts_with('\\')
    && !path.ends_with('\\')
    && !path
      .chars()
      .any(|character| character == '/' || character.is_uppercase())
}

/// Rejects a path whose components the engine cannot address.
///
/// `original` is reported rather than the rewritten form, so the error names what the caller passed.
fn validate_components(normalized: &str, original: &str) -> XrfResult<()> {
  if normalized.is_empty()
    || normalized
      .split('\\')
      .any(|part| part.is_empty() || matches!(part, "." | ".."))
  {
    return Err(XrfError::new_asset_error(format!(
      "invalid X-Ray logical path: {original}"
    )));
  }

  Ok(())
}

/// Normalizes a mount's logical base, where empty means the whole root rather than an invalid path.
///
/// The one place that exception lives. A mount and the plan that describes it both have to agree on it, and
/// [`normalize`] cannot answer it: an empty path is exactly what it rejects.
///
/// # Errors
///
/// Returns an error when a non-empty base is not a valid logical path.
pub(crate) fn normalize_base(base: &str) -> XrfResult<String> {
  if base.is_empty() {
    return Ok(String::new());
  }

  Ok(normalize(base)?.into_owned())
}

/// Converts a root-relative host path into the canonical X-Ray logical path used for indexing.
///
/// Named for the domain it crosses: the input is a host path fragment, the output an engine identity.
pub(crate) fn normalize_host_relative(path: &Path) -> XrfResult<String> {
  let path: &str = path
    .to_str()
    .ok_or_else(|| XrfError::new_asset_error(format!("directory asset path is not valid UTF-8: {}", path.display())))?;

  Ok(normalize(path)?.into_owned())
}

/// Converts an engine identity into a host path relative to a source root.
///
/// The deliberate crossing out of the engine domain, and the inverse of [`normalize_host_relative`]. Built from
/// components so the platform inserts its own separator: a `\`-joined literal is one component on Linux, which is how a
/// created file lands with a backslash in its name instead of in a subdirectory.
pub(crate) fn to_host_relative(logical_path: &str) -> PathBuf {
  logical_path.split('\\').collect()
}

pub(crate) fn join(prefix: &str, path: &str) -> XrfResult<String> {
  let joined: Cow<str> = match (prefix.is_empty(), path.is_empty()) {
    (true, true) => normalize("")?,
    (true, false) => normalize(path)?,
    (false, true) => normalize(prefix)?,
    (false, false) => Cow::Owned(normalize(&format!("{prefix}\\{path}"))?.into_owned()),
  };

  Ok(joined.into_owned())
}

/// Whether a logical path carries `extension`, compared without case.
pub(crate) fn has_extension(path: &str, extension: &str) -> bool {
  path.len() > extension.len()
    && path
      .get(path.len() - extension.len()..)
      .is_some_and(|tail| tail.eq_ignore_ascii_case(extension))
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use super::{XrayLogicalPath, has_extension, join, normalize, normalize_host_relative, to_host_relative};

  #[test]
  fn does_not_treat_a_bare_extension_as_a_named_asset() {
    assert!(!has_extension(".ltx", ".ltx"));
  }

  #[test]
  fn normalizes_case_and_separators() {
    assert_eq!(normalize("Configs/System.LTX").expect("valid"), "configs\\system.ltx");
    assert_eq!(normalize("\\configs\\").expect("valid"), "configs");
  }

  #[test]
  fn rejects_traversal_and_empty_components() {
    assert!(normalize("configs\\..\\textures").is_err());
    assert!(normalize("configs\\\\system.ltx").is_err());
    assert!(normalize("").is_err());
  }

  #[test]
  fn joins_either_side_being_empty() {
    assert_eq!(join("configs", "system.ltx").expect("valid"), "configs\\system.ltx");
    assert_eq!(join("", "system.ltx").expect("valid"), "system.ltx");
    assert_eq!(join("configs", "").expect("valid"), "configs");
  }

  #[test]
  fn converts_an_identity_into_a_host_relative_path() {
    // Built from components, so the platform inserts its own separator. A `\`-joined literal is one filename on Linux,
    // which would create a file called `configs\system.ltx` instead of one inside `configs`.
    assert_eq!(
      to_host_relative("configs\\system.ltx"),
      Path::new("configs").join("system.ltx")
    );
    assert_eq!(to_host_relative("system.ltx"), Path::new("system.ltx"));
    assert_eq!(
      normalize_host_relative(&to_host_relative("configs\\weapons\\ak74.ltx")).expect("valid"),
      "configs\\weapons\\ak74.ltx",
      "crossing to a host path and back is lossless"
    );
  }

  #[test]
  fn converts_a_host_relative_path_into_an_identity() {
    assert_eq!(
      normalize_host_relative(&PathBuf::from("Configs").join("System.ltx")).expect("valid"),
      "configs\\system.ltx"
    );
  }

  #[test]
  fn normalizes_a_typed_path_on_construction() {
    let path: XrayLogicalPath = XrayLogicalPath::new("Configs/System.LTX").expect("valid");

    assert_eq!(path.as_str(), "configs\\system.ltx");
    assert_eq!(path.to_string(), "configs\\system.ltx");
    assert!(XrayLogicalPath::new("configs\\..\\system.ltx").is_err());
  }

  #[test]
  fn answers_its_parent_and_name() {
    let nested: XrayLogicalPath = XrayLogicalPath::new("configs\\weapons\\w_ak74.ltx").expect("valid");

    assert_eq!(nested.file_name(), "w_ak74.ltx");
    assert_eq!(
      nested.parent().map(|parent| parent.as_str().to_string()),
      Some(String::from("configs\\weapons"))
    );

    let top_level: XrayLogicalPath = XrayLogicalPath::new("system.ltx").expect("valid");

    assert_eq!(top_level.file_name(), "system.ltx");
    assert_eq!(top_level.parent(), None);
  }

  #[test]
  fn matches_an_extension_without_case_and_a_prefix_by_component() {
    let path: XrayLogicalPath = XrayLogicalPath::new("configs\\system.ltx").expect("valid");

    assert!(path.has_extension(".LTX"));
    assert!(path.is_under("configs").expect("valid prefix"));
    assert!(!path.is_under("configs_backup").expect("valid prefix"));
  }

  #[test]
  fn joins_below_itself() {
    let directory: XrayLogicalPath = XrayLogicalPath::new("configs\\weapons").expect("valid");

    assert_eq!(
      directory.join("w_ak74.ltx").expect("valid").as_str(),
      "configs\\weapons\\w_ak74.ltx"
    );
  }

  #[test]
  fn answers_the_same_on_every_platform_where_std_path_would_not() {
    // The portability contract. On Linux `std::path` treats `configs\weapons\w_ak74.ltx` as one component.
    let path: XrayLogicalPath = XrayLogicalPath::new("configs\\weapons\\w_ak74.ltx").expect("valid");
    let as_host_path: &Path = Path::new(path.as_str());

    assert_eq!(path.file_name(), "w_ak74.ltx");
    assert_eq!(
      path.parent().expect("nested path has a parent").as_str(),
      "configs\\weapons"
    );
    assert!(path.is_under("configs").expect("valid prefix"));

    // Separators come in either way and leave as `\`, so a host path built with `/` addresses the same asset.
    assert_eq!(
      XrayLogicalPath::new("configs/weapons/w_ak74.ltx").expect("valid"),
      path,
      "a forward-slash reference names the same engine asset"
    );
    assert_eq!(
      normalize_host_relative(&PathBuf::from("configs").join("weapons").join("w_ak74.ltx")).expect("valid"),
      path.as_str(),
      "a host relative path indexes to the same identity whichever separator the platform used"
    );

    // Left as a marker of why the type exists: on Windows these agree, on Linux they do not.
    assert_eq!(as_host_path.extension().and_then(|value| value.to_str()), Some("ltx"));
  }
}
