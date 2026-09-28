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
  /// # Errors
  ///
  /// Returns an error when the root exists but cannot be planned.
  pub fn with_root(self, label: impl Into<String>, root: impl AsRef<Path>) -> XrfResult<Self> {
    self.with_root_mode(label, root, XrayMountMode::Auto)
  }

  /// Searches one root, read the way the caller says rather than through the default.
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
  /// # Errors
  ///
  /// Returns an error when a mount plan cannot be applied to the VFS.
  pub fn mount_into(&self, vfs: &mut XrayVfs) -> XrfResult<Vec<XrayProbeStep>> {
    let mut steps: Vec<XrayProbeStep> = Vec::with_capacity(self.steps.len());

    for step in &self.steps {
      let mounts: Vec<XrayMountId> = vfs.mount_plan(&step.plan)?;

      steps.push(XrayProbeStep::planned(
        &step.label,
        XrayLookupScope::only(mounts),
        vfs.list_skipped_mounts_of(&step.plan),
      ));
    }

    Ok(steps)
  }

  /// Forgets the recorded failures among the sources every step names, so mounting the plan again tries each of them.
  pub fn forget_skipped_in(&self, vfs: &mut XrayVfs) {
    for step in &self.steps {
      vfs.forget_skipped_mounts_of(&step.plan);
    }
  }

  /// The steps [`Self::mount_into`] would return, where every source the plan names is settled already; `None` where
  /// one is not.
  ///
  /// A source that failed to open is settled by its recorded failure, which the step reports as skipped.
  pub fn find_mounted(&self, vfs: &XrayVfs) -> Option<Vec<XrayProbeStep>> {
    self
      .steps
      .iter()
      .map(|step| {
        vfs.find_mounted_plan(&step.plan).map(|mounts| {
          XrayProbeStep::planned(
            &step.label,
            XrayLookupScope::only(mounts),
            vfs.list_skipped_mounts_of(&step.plan),
          )
        })
      })
      .collect()
  }
}
