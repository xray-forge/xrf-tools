use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;
use xrf_utils::format_path;

use crate::FsgameFile;
use crate::mount::xray_mount_mode::XrayMountMode;
use crate::mount::xray_mount_plan::XrayMountPlan;
use crate::mount::xray_probe_plan::XrayProbePlan;
use crate::vfs::XrayVfs;

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

  /// Each installation these roots sit in, asset's first, once each: a root holding `fsgame.ltx`, or the nearest
  /// ancestor that does.
  pub fn list_installations(&self) -> Vec<PathBuf> {
    let mut installations: Vec<PathBuf> = Vec::new();

    for path in self.asset.iter().chain(self.roots.iter().map(|root| &root.path)) {
      if let Some(installation) = path
        .ancestors()
        .find(|candidate| candidate.join(FsgameFile::FILE_NAME).is_file())
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

  /// The mounts these roots mean, in search order.
  ///
  /// # Errors
  ///
  /// Returns an error when a root cannot be planned — `Installation` on a path declaring none, or an
  /// `fsgame.ltx` that cannot be read.
  pub fn to_mount_plan(&self) -> XrfResult<XrayMountPlan> {
    let mut plan: XrayMountPlan = match &self.asset {
      Some(asset) => XrayMountPlan::implied(asset)?,
      None => XrayMountPlan::new(),
    };

    for root in &self.roots {
      plan = plan.behind(root.mode.plan(&root.path)?);
    }

    Ok(plan)
  }

  /// The ordered probe steps these roots mean.
  ///
  /// # Errors
  ///
  /// Returns an error when the asset's own sources or one of the roots cannot be planned.
  pub fn to_probe_plan(&self) -> XrfResult<XrayProbePlan> {
    let mut plan: XrayProbePlan = XrayProbePlan::new();

    if let Some(asset) = &self.asset {
      plan = plan.with_asset(Path::new(asset))?;
    }

    for root in &self.roots {
      plan = plan.with_root_mode(format_path(&root.path).to_string(), &root.path, root.mode)?;
    }

    Ok(plan)
  }

  /// Mount these roots and hand back the result.
  ///
  /// # Errors
  ///
  /// Returns an error when the spec cannot be planned or a planned source cannot be mounted.
  pub fn open(&self) -> XrfResult<XrayVfs> {
    XrayVfs::from_plan(&self.to_mount_plan()?)
  }
}
