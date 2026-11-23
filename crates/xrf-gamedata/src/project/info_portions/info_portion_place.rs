use std::path::PathBuf;

/// Where an info portion is named: the file a finding reports, and what in it names the portion, such as a section's
/// key, a dialog or a spawned object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InfoPortionPlace {
  pub(crate) path: PathBuf,
  pub(crate) context: String,
}
