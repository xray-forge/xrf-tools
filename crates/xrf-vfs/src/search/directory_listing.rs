use crate::XrayAsset;

/// What sits directly inside one logical directory.
#[derive(Clone, Debug, Default)]
pub struct XrayDirectoryListing {
  /// Names of the directories directly inside, sorted, without their parent path.
  pub directories: Vec<String>,
  /// Winning entries sitting directly inside, sorted by logical path.
  pub files: Vec<XrayAsset>,
}

impl XrayDirectoryListing {
  /// Whether the directory holds nothing the current scope can see.
  pub fn is_empty(&self) -> bool {
    self.directories.is_empty() && self.files.is_empty()
  }
}
