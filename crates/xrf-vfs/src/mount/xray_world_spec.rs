use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;

use crate::mount::xray_mount_mode::XrayMountMode;
use crate::mount::xray_mount_plan::XrayMountPlan;
use crate::mount::xray_probe_plan::XrayProbePlan;
use crate::vfs::XrayVfs;

/// One root a world is assembled from, and how that root becomes mounts.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayWorldRoot {
  pub path: String,
  /// How this path becomes mounts. `Auto` unless the caller says otherwise.
  #[serde(default)]
  pub mode: XrayMountMode,
}

impl XrayWorldRoot {
  pub fn new(path: impl Into<String>, mode: XrayMountMode) -> Self {
    Self {
      path: path.into(),
      mode,
    }
  }
}

/// Where a world is: an optional subject asset, then ordered roots.
///
/// The one way every surface names a world, so `--source` on a command, a setting in the app, and an
/// editor session all say the same thing. What sits *inside* the world is a separate question that
/// stays with each domain — a dialog layout and a translations layout disagree, and a spawn file has
/// no answer at all.
///
/// Several roots is layering, which is how modding actually works: a loose gamedata tree in front of
/// an installation. Search order is declaration order, and the first mount holding a path wins.
///
/// Callers do not assemble mounts from this themselves. They hand it to whatever owns mounting and
/// receive a world or a probe, so there is one place that decides what a spec means.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayWorldSpec {
  /// Asset whose own X-Ray root and installation are searched first, when the world is centred on one.
  ///
  /// This is what finds a texture shipped beside a model rather than in the shared tree.
  pub asset: Option<String>,
  /// Roots searched after the asset's own, in the order given.
  pub roots: Vec<XrayWorldRoot>,
}

impl XrayWorldSpec {
  /// A world of one root.
  pub fn root(path: impl Into<String>, mode: XrayMountMode) -> Self {
    Self {
      asset: None,
      roots: vec![XrayWorldRoot::new(path, mode)],
    }
  }

  /// A world of several roots, in search order.
  pub fn roots(roots: impl IntoIterator<Item = XrayWorldRoot>) -> Self {
    Self {
      asset: None,
      roots: roots.into_iter().collect(),
    }
  }

  /// The same world, centred on an asset when it does not already name one.
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

  /// Whether this spec names nowhere at all.
  pub fn is_empty(&self) -> bool {
    self.asset.is_none() && self.roots.is_empty()
  }

  /// The mounts this spec means, in search order.
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

  /// The ordered probe steps this spec means.
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
      plan = plan.with_root(root.path.clone(), &root.path)?;
    }

    Ok(plan)
  }

  /// Mount this world and hand back the result.
  ///
  /// # Errors
  ///
  /// Returns an error when the spec cannot be planned or a planned source cannot be mounted.
  pub fn open(&self) -> XrfResult<XrayVfs> {
    XrayVfs::from_plan(&self.to_mount_plan()?)
  }
}
