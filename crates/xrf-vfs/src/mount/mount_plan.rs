use xrf_error::XrfResult;
use xrf_utils::format_path;

use crate::source::{XrayArchiveSource, XrayDirectorySource};
use crate::{XrayMountId, XrayMountPlan, XrayPlannedMount, XraySkippedMount, XraySourceKind, XrayVfs};

impl XrayVfs {
  /// Mounts each planned source that can be opened, in plan order, behind the mounts already present.
  pub fn mount_plan(&mut self, plan: &XrayMountPlan) -> XrfResult<Vec<XrayMountId>> {
    let mut mounted: Vec<XrayMountId> = Vec::with_capacity(plan.len());

    for planned in plan.get_mounts() {
      if self.planned_mount(&planned.path, planned.kind).is_none()
        && self.skipped_mount(&planned.path, planned.kind).is_some()
      {
        continue;
      }

      match mount_one(self, planned) {
        Ok(id) => {
          self.record_origin(id, &planned.origin);
          mounted.push(id);
        }
        Err(error) => {
          log::warn!(
            "Skipping planned mount {} at {}: {error}",
            planned.origin,
            format_path(&planned.path)
          );

          self.record_skipped(
            planned.kind,
            XraySkippedMount {
              origin: planned.origin.clone(),
              path: planned.path.clone(),
              reason: error.to_string(),
            },
          );
        }
      }
    }

    Ok(mounted)
  }

  /// The mounts a plan produced, in plan order, where every source it names is settled already; `None` where one is
  /// not, which only [`Self::mount_plan`] can settle.
  pub fn find_mounted_plan(&self, plan: &XrayMountPlan) -> Option<Vec<XrayMountId>> {
    let mut mounted: Vec<XrayMountId> = Vec::with_capacity(plan.len());

    for planned in plan.get_mounts() {
      match self.planned_mount(&planned.path, planned.kind) {
        Some(id) => mounted.push(id),
        None => {
          self.skipped_mount(&planned.path, planned.kind)?;
        }
      }
    }

    Some(mounted)
  }

  /// The recorded failures among the sources a plan names, in plan order.
  pub fn list_skipped_mounts_of(&self, plan: &XrayMountPlan) -> Vec<XraySkippedMount> {
    plan
      .get_mounts()
      .iter()
      .filter(|planned| self.planned_mount(&planned.path, planned.kind).is_none())
      .filter_map(|planned| self.skipped_mount(&planned.path, planned.kind))
      .cloned()
      .collect()
  }
}

fn mount_one(vfs: &mut XrayVfs, planned: &XrayPlannedMount) -> XrfResult<XrayMountId> {
  // Checked before constructing the source, because constructing it is what indexes the tree or the name table.
  if let Some(existing) = vfs.planned_mount(&planned.path, planned.kind) {
    log::debug!(
      "Reusing mount {existing:?} for already-mounted {} at {}",
      planned.origin,
      format_path(&planned.path)
    );

    return Ok(existing);
  }

  let id: XrayMountId = match planned.kind {
    XraySourceKind::Archive => vfs.mount(&planned.base, Box::new(XrayArchiveSource::read(&planned.path)?))?,
    XraySourceKind::Directory => vfs.mount(
      &planned.base,
      Box::new(XrayDirectorySource::read_ignoring(&planned.path, &planned.ignored)?),
    )?,
  };

  vfs.record_planned(planned.path.clone(), id);

  Ok(id)
}
