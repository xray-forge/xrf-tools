use std::borrow::Cow;

/// A section name lowercased, as `CInifile::Load` stores headers and `CInifile::r_section` looks them up.
///
/// Borrowed when already lowercase, as nearly every shipped name is.
pub fn fold_section_name(name: &str) -> Cow<'_, str> {
  if name.chars().any(char::is_uppercase) {
    Cow::Owned(name.to_lowercase())
  } else {
    Cow::Borrowed(name)
  }
}
