use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;

use crate::mount::xray_mount_mode::XrayMountMode;
use crate::mount::xray_mount_plan::XrayMountPlan;
use crate::mount::xray_probe_plan::XrayProbePlan;
use crate::vfs::XrayVfs;

/// One place to read from, and how that place becomes mounts.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayRoot {
  pub path: String,
  /// How this path becomes mounts. `Auto` unless the caller says otherwise.
  #[serde(default)]
  pub mode: XrayMountMode,
}

impl XrayRoot {
  pub fn new(path: impl Into<String>, mode: XrayMountMode) -> Self {
    Self {
      path: path.into(),
      mode,
    }
  }
}

/// Everywhere a caller wants read: an optional subject asset, then ordered roots.
///
/// The one way every surface says where to read from, so `--source` on a command, a setting in the
/// app, and an editor session all name the same thing. What sits *inside* those roots is a separate
/// question that stays with each domain — a dialog layout and a translations layout disagree about it,
/// and a spawn file has no answer at all.
///
/// Several roots means layering, which is how modding actually works: a loose gamedata tree in front
/// of an installation. Search order is declaration order, and the first mount holding a path wins.
///
/// Callers do not assemble mounts from this themselves. They hand it to whatever owns mounting and
/// receive a VFS or a probe back, so one place decides what a declaration means.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayRoots {
  /// Asset whose own X-Ray root and installation are searched first, when the read is centred on one.
  ///
  /// This is what finds a texture shipped beside a model rather than in the shared tree.
  pub asset: Option<String>,
  /// Roots searched after the asset's own, in the order given.
  pub roots: Vec<XrayRoot>,
}

impl XrayRoots {
  /// One root, read the given way.
  pub fn one(path: impl Into<String>, mode: XrayMountMode) -> Self {
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
  ///
  /// Lets a command fill in the subject it knows about while leaving a caller free to name a different
  /// one, and the result is what travels back to the frontend — so a later read searches what the open
  /// searched.
  pub fn centred_on(&self, asset: Option<&Path>) -> Self {
    Self {
      asset: self
        .asset
        .clone()
        .or_else(|| asset.map(|path| path.display().to_string())),
      roots: self.roots.clone(),
    }
  }

  /// Whether this names nowhere at all.
  pub fn is_empty(&self) -> bool {
    self.asset.is_none() && self.roots.is_empty()
  }

  /// Name these roots for a log line or an error message.
  ///
  /// On the type because roots have no single path to print and every surface reporting on one had
  /// started writing its own join.
  pub fn describe(&self) -> String {
    if self.roots.is_empty() {
      return match &self.asset {
        Some(asset) => asset.clone(),
        None => String::from("<no roots>"),
      };
    }

    self
      .roots
      .iter()
      .map(|root| root.path.clone())
      .collect::<Vec<String>>()
      .join(", ")
  }

  /// The mounts these roots mean, in search order.
  ///
  /// For a tool that lists and reads a whole tree. `XrayMountPlan::behind` dedupes by path, so a
  /// fallback root that happens to be the tree the asset already implied is not mounted twice.
  ///
  /// # Errors
  ///
  /// Returns an error when a root cannot be planned — `Installation` on a path declaring none, or an
  /// `fsgame.ltx` that cannot be read.
  pub fn to_mount_plan(&self) -> XrfResult<XrayMountPlan> {
    let mut plan: XrayMountPlan = match &self.asset {
      Some(asset) => XrayMountPlan::implied(Path::new(asset))?,
      None => XrayMountPlan::new(),
    };

    for root in &self.roots {
      plan = plan.behind(root.mode.plan(&root.path)?);
    }

    Ok(plan)
  }

  /// The ordered probe steps these roots mean.
  ///
  /// For a per-asset lookup that has to report which step answered. Each root is labelled with its own
  /// path, because that is the string a failed lookup lists as searched.
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
      plan = plan.with_root_mode(root.path.clone(), &root.path, root.mode)?;
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
