use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;
use xrf_utils::format_path;

use crate::mount::xray_mount_mode::XrayMountMode;
use crate::{FsgameFile, XraySearchPlan};

/// One place to read from, and how that place becomes mounts.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayRoot {
  /// Native host address, retained without rendering it as text.
  pub path: PathBuf,
  /// How this path becomes mounts. `Auto` unless the caller says otherwise.
  #[serde(default)]
  pub mode: XrayMountMode,
}

impl XrayRoot {
  pub fn new(path: PathBuf, mode: XrayMountMode) -> Self {
    Self { path, mode }
  }
}

/// Everywhere a caller wants read: an optional subject asset, then ordered roots.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayRoots {
  /// Native asset address whose own X-Ray root and installation are searched first, when the read is centred on one.
  pub asset: Option<PathBuf>,
  /// Roots searched after the asset's own, in the order given.
  pub roots: Vec<XrayRoot>,
}

impl XrayRoots {
  /// One root, read the given way.
  pub fn one(path: PathBuf, mode: XrayMountMode) -> Self {
    Self {
      asset: None,
      roots: vec![XrayRoot::new(path, mode)],
    }
  }

  /// Several roots, in search order.
  pub fn new(roots: impl IntoIterator<Item = XrayRoot>) -> Self {
    Self {
      asset: None,
      roots: roots.into_iter().collect(),
    }
  }

  /// The same roots, centred on an asset when it does not already name one.
  pub fn centred_on(&self, asset: Option<&Path>) -> Self {
    Self {
      asset: self.asset.clone().or_else(|| asset.map(Path::to_path_buf)),
      roots: self.roots.clone(),
    }
  }

  /// Each installation these roots sit in, asset's first, once each: a root holding an fsgame file, or the nearest
  /// ancestor that does.
  pub fn list_installations(&self) -> Vec<PathBuf> {
    let mut installations: Vec<PathBuf> = Vec::new();

    for path in self.asset.iter().chain(self.roots.iter().map(|root| &root.path)) {
      if let Some(installation) = path.ancestors().find(|candidate| FsgameFile::find(candidate).is_some())
        && !installations.iter().any(|known| known == installation)
      {
        installations.push(installation.to_path_buf());
      }
    }

    installations
  }

  /// Whether this names nowhere at all.
  pub fn is_empty(&self) -> bool {
    self.asset.is_none() && self.roots.is_empty()
  }

  /// Name these roots for a log line or an error message.
  pub fn describe(&self) -> String {
    if self.roots.is_empty() {
      return match &self.asset {
        Some(asset) => format_path(asset).to_string(),
        None => String::from("<no roots>"),
      };
    }

    self
      .roots
      .iter()
      .map(|root| format_path(&root.path).to_string())
      .collect::<Vec<String>>()
      .join(", ")
  }

  /// The ordered search these roots mean: the asset's own root and installation, then each root, named by its path.
  ///
  /// # Errors
  ///
  /// Returns an error when the asset's own sources or one of the roots cannot be planned — `Installation` on a path
  /// declaring none, or an fsgame file that cannot be read. A root that is not there is an empty step reported as
  /// skipped, not an error.
  pub fn to_search_plan(&self) -> XrfResult<XraySearchPlan> {
    let mut plan: XraySearchPlan = XraySearchPlan::new();

    if let Some(asset) = &self.asset {
      plan = plan.with_asset(Path::new(asset))?;
    }

    for root in &self.roots {
      plan = plan.with_root_mode(format_path(&root.path).to_string(), &root.path, root.mode)?;
    }

    Ok(plan)
  }
}
