use std::borrow::Cow;

/// A section name as the engine stores and looks it up: lowercased (`CInifile::Load` calls `xr_strlwr` on every header,
/// `r_section` on every query). Borrowed when the name is already lowercase, as nearly every name in shipped configs is.
pub fn fold_section_name(name: &str) -> Cow<'_, str> {
  if name.chars().any(char::is_uppercase) {
    Cow::Owned(name.to_lowercase())
  } else {
    Cow::Borrowed(name)
  }
}
