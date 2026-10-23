use std::path::{Path, PathBuf};

use xrf_error::XrfResult;

use crate::mount::XrayMountPlan;
use crate::{
  XrayLookupScope, XrayMountId, XrayMountMode, XrayRootProbe, XraySearch, XraySearchStep, XraySkippedMount,
  XraySourceKind, XrayVfs,
};

/// One declared place to search, before it has been mounted.
#[derive(Clone, Debug)]
struct PlannedSearchStep {
  label: String,
  plan: XrayMountPlan,
  /// A root the caller named that is not there, which the step reports as skipped instead of searching.
  missing: Option<XraySkippedMount>,
}

impl PlannedSearchStep {
  fn new(label: impl Into<String>, plan: XrayMountPlan) -> Self {
    Self {
      label: label.into(),
      plan,
      missing: None,
    }
  }

  /// The step this declaration mounted as, carrying every source it could not open.
  fn to_step(&self, vfs: &XrayVfs, mounts: Vec<XrayMountId>) -> XraySearchStep {
    let mut skipped: Vec<XraySkippedMount> = vfs.list_skipped_mounts_of(&self.plan);

    skipped.extend(self.missing.iter().cloned());

    XraySearchStep::planned(&self.label, XrayLookupScope::only(mounts), skipped)
  }
}

/// An ordered search declared before anything is mounted, so the order survives the mounting.
///
/// The one way a path becomes mounts: [`Self::open`] gives a tool a VFS of its own, and [`Self::mount_into`] adds the
/// steps to a VFS someone else holds.
#[derive(Clone, Debug, Default)]
pub struct XraySearchPlan {
  steps: Vec<PlannedSearchStep>,
}

impl XraySearchPlan {
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
  /// Returns an error when an installation is found but its fsgame file cannot be read, decoded, or parsed.
  pub fn with_asset(mut self, asset: impl AsRef<Path>) -> XrfResult<Self> {
    let asset: &Path = asset.as_ref();

    self
      .steps
      .push(PlannedSearchStep::new(Self::ASSET_STEP, XrayMountPlan::implied(asset)?));
    self.steps.push(PlannedSearchStep::new(
      Self::INSTALLATION_STEP,
      XrayMountPlan::implied_install(asset)?,
    ));

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
  /// Returns an error when the root exists but cannot be planned, such as [`XrayMountMode::Installation`] on a path
  /// declaring none. A root that is not there is an empty step that reports it as skipped.
  pub fn with_root_mode(
    mut self,
    label: impl Into<String>,
    root: impl AsRef<Path>,
    mode: XrayMountMode,
  ) -> XrfResult<Self> {
    let root: PathBuf = root.as_ref().to_path_buf();

    // A volume named as a root is a root: dropping it for not being a directory would silently search nothing,
    // which reads exactly like an archive whose entries have all gone missing.
    if root.is_dir() || XrayRootProbe::is_volume(&root) {
      self.steps.push(PlannedSearchStep::new(label, mode.plan(&root)?));
    } else {
      self.steps.push(PlannedSearchStep {
        missing: Some(XraySkippedMount {
          origin: mode.to_string(),
          reason: String::from("neither a directory nor an archive volume is there to mount"),
          path: root,
        }),
        ..PlannedSearchStep::new(label, XrayMountPlan::new())
      });
    }

    Ok(self)
  }

  /// This plan with every loose directory it mounts omitting the logical prefixes `ignored` names; archive volumes are
  /// unaffected, since nothing writes work in progress into a volume.
  ///
  /// # Errors
  ///
  /// Returns an error when a prefix is not a valid X-Ray logical path.
  pub fn ignoring(mut self, ignored: &[String]) -> XrfResult<Self> {
    for step in &mut self.steps {
      step.plan = std::mem::take(&mut step.plan).ignoring(ignored)?;
    }

    Ok(self)
  }

  /// This plan keeping only the sources of one kind, step by step: an installation's volume sets apart from the loose
  /// trees overriding them, decided by one reading of its fsgame file rather than two that could disagree.
  pub fn of_kind(&self, kind: XraySourceKind) -> Self {
    Self {
      steps: self
        .steps
        .iter()
        .map(|step| PlannedSearchStep {
          plan: step.plan.of_kind(kind),
          ..step.clone()
        })
        .collect(),
    }
  }

  /// Whether nothing has been declared to search.
  pub fn is_empty(&self) -> bool {
    self.steps.is_empty()
  }

  /// How many sources the steps plan to mount, before any is opened: none for a plan whose every step searches
  /// nothing, as one keeping the volumes of a tree holding only loose files does.
  pub fn count_sources(&self) -> usize {
    self.steps.iter().map(|step| step.plan.len()).sum()
  }

  /// A search over a VFS of its own, mounting every declared step.
  ///
  /// # Errors
  ///
  /// As [`Self::mount_into`].
  pub fn open(&self) -> XrfResult<XraySearch> {
    self.open_in(XrayVfs::new())
  }

  /// A search over `vfs`, a snapshot built the way the caller needs it (a cache policy, a read trace), with every
  /// declared step mounted into it.
  ///
  /// # Errors
  ///
  /// As [`Self::mount_into`].
  pub fn open_in(&self, mut vfs: XrayVfs) -> XrfResult<XraySearch> {
    let steps: Vec<XraySearchStep> = self.mount_into(&mut vfs)?;

    Ok(XraySearch::new(vfs).with_steps(steps))
  }

  /// Mounts every declared step into a VFS and returns the steps in search order.
  ///
  /// # Errors
  ///
  /// Returns an error when a mount plan cannot be applied to the VFS. A source that fails to open is logged, recorded
  /// on the VFS and reported by its step as skipped, so a partly readable installation still resolves what it can.
  pub fn mount_into(&self, vfs: &mut XrayVfs) -> XrfResult<Vec<XraySearchStep>> {
    let mut steps: Vec<XraySearchStep> = Vec::with_capacity(self.steps.len());

    for step in &self.steps {
      let mounts: Vec<XrayMountId> = vfs.mount_plan(&step.plan)?;

      steps.push(step.to_step(vfs, mounts));
    }

    Ok(steps)
  }

  /// Forgets the recorded failures among the sources every step names, so mounting the plan again tries each of them.
  pub fn forget_skipped_in(&self, vfs: &mut XrayVfs) {
    for step in &self.steps {
      vfs.forget_skipped_mounts_of(&step.plan);
    }
  }

  /// Whether a source this plan names failed to open in `vfs`, which mounting it again after
  /// [`Self::forget_skipped_in`] would try once more. A root that is not there is not a failure to open.
  pub fn has_failed_in(&self, vfs: &XrayVfs) -> bool {
    self
      .steps
      .iter()
      .any(|step| !vfs.list_skipped_mounts_of(&step.plan).is_empty())
  }

  /// The steps [`Self::mount_into`] would return, where every source the plan names is settled already; `None` where
  /// one is not.
  ///
  /// A source that failed to open is settled by its recorded failure, which the step reports as skipped.
  pub fn find_mounted(&self, vfs: &XrayVfs) -> Option<Vec<XraySearchStep>> {
    self
      .steps
      .iter()
      .map(|step| {
        vfs
          .find_mounted_plan(&step.plan)
          .map(|mounts| step.to_step(vfs, mounts))
      })
      .collect()
  }
}
