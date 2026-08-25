use std::path::{Path, PathBuf};

use xrf_error::XrfResult;

use crate::{XrayLookupScope, XrayMountId, XrayMountMode, XrayMountPlan, XrayProbeStep, XrayVfs};

/// One declared place to search, before it has been mounted.
#[derive(Clone, Debug)]
struct PlannedProbeStep {
  label: String,
  plan: XrayMountPlan,
}

/// An ordered search declared before anything is mounted, so the order survives the mounting.
///
/// A probe's value is its order, and the order is decided by the caller's intent rather than by which source happens to
/// open first: a viewer searches an asset's own tree before the project it belongs to, precisely so a loose file beside a
/// model wins over the same name in gamedata. Declaring the steps and mounting them in one pass is what keeps that
/// intact — mounting as each root is discovered produces whatever order discovery happened in.
///
/// Mounting is idempotent, because [`XrayVfs`] reuses a mount for a planned path it already holds. A viewer that opens
/// fifty models under one root therefore pays for one index, not fifty, which is why a probe may be planned per asset.
#[derive(Clone, Debug, Default)]
pub struct XrayProbePlan {
  steps: Vec<PlannedProbeStep>,
}

impl XrayProbePlan {
  /// Label of the step covering an asset's own X-Ray root.
  pub const ASSET_STEP: &'static str = "asset root";
  /// Label of the step covering the installation an asset sits in.
  pub const INSTALLATION_STEP: &'static str = "installation";

  pub fn new() -> Self {
    Self::default()
  }

  /// Searches an asset's own X-Ray root, then the installation containing it.
  ///
  /// Two steps rather than one, because they answer differently: the first is the loose tree the asset sits in, the
  /// second is that tree's game install including its volumes. Either may derive nothing, which leaves a step that
  /// selects no mount and is skipped at lookup time.
  ///
  /// # Errors
  ///
  /// Returns an error when an installation is found but its `fsgame.ltx` cannot be read, decoded, or parsed.
  pub fn with_asset(mut self, asset: impl AsRef<Path>) -> XrfResult<Self> {
    let asset: &Path = asset.as_ref();

    self.steps.push(PlannedProbeStep {
      label: Self::ASSET_STEP.to_string(),
      plan: XrayMountPlan::implied(asset)?,
    });
    self.steps.push(PlannedProbeStep {
      label: Self::INSTALLATION_STEP.to_string(),
      plan: XrayMountPlan::implied_install(asset)?,
    });

    Ok(self)
  }

  /// Searches one root, named by the caller because only the caller knows what it means to a reader.
  ///
  /// Planned through [`XrayMountMode::Auto`], so a root the user picked is treated as whatever it is: an installation
  /// with its volumes, a bare volume set, or a loose tree. A viewer pointed at `<install>\db` otherwise mounts the
  /// volumes as files and finds no assets at all.
  ///
  /// A path that is neither a directory nor an archive volume plans nothing rather than failing: an unconfigured
  /// project root is an ordinary state of a viewer, not an error to report.
  ///
  /// # Errors
  ///
  /// Returns an error when the root exists but cannot be planned.
  pub fn with_root(self, label: impl Into<String>, root: impl AsRef<Path>) -> XrfResult<Self> {
    self.with_root_mode(label, root, XrayMountMode::Auto)
  }

  /// Searches one root, read the way the caller says rather than through the default.
  ///
  /// The same step as [`Self::with_root`], for a caller carrying a mode per root — an `XrayRoots` names
  /// one each, so a loose gamedata tree and the installation behind it can be read differently within
  /// one search.
  ///
  /// A path that is neither a directory nor an archive volume plans nothing rather than failing, for the
  /// reason above: an unconfigured project root is an ordinary state of a viewer, not an error to report.
  ///
  /// # Errors
  ///
  /// Returns an error when the root exists but cannot be planned.
  pub fn with_root_mode(
    mut self,
    label: impl Into<String>,
    root: impl AsRef<Path>,
    mode: XrayMountMode,
  ) -> XrfResult<Self> {
    let root: PathBuf = root.as_ref().to_path_buf();

    self.steps.push(PlannedProbeStep {
      label: label.into(),
      // A volume named as a root is a root: dropping it for not being a directory would silently search nothing,
      // which reads exactly like an archive whose entries have all gone missing.
      plan: if root.is_dir() || XrayMountPlan::is_volume(&root) {
        mode.plan(&root)?
      } else {
        XrayMountPlan::new()
      },
    });

    Ok(self)
  }

  /// Whether nothing has been declared to search.
  pub fn is_empty(&self) -> bool {
    self.steps.is_empty()
  }

  /// Mounts every declared step into a VFS and returns the steps in search order.
  ///
  /// Returns the steps rather than a probe so that mounting, which needs the VFS mutably, finishes before the probe
  /// borrows it. Hand them to [`crate::XrayProbe::with_steps`].
  ///
  /// A step whose plan is empty, or whose sources all fail to open, still becomes a step: it selects no mount, so it is
  /// skipped rather than answering, and it stays visible to a caller reporting on what was searched.
  ///
  /// # Errors
  ///
  /// Returns an error when a mount plan cannot be applied to the VFS.
  pub fn mount_into(&self, vfs: &mut XrayVfs) -> XrfResult<Vec<XrayProbeStep>> {
    let mut steps: Vec<XrayProbeStep> = Vec::with_capacity(self.steps.len());

    for step in &self.steps {
      let mounts: Vec<XrayMountId> = vfs.mount_plan(&step.plan)?;

      steps.push(XrayProbeStep::new(&step.label, XrayLookupScope::only(mounts)));
    }

    Ok(steps)
  }
}
