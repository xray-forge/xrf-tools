use xrf_error::XrfResult;
use xrf_utils::format_path;

use crate::source::{XrayArchiveSource, XrayDirectorySource};
use crate::{XrayMountId, XrayMountPlan, XrayPlannedMount, XraySkippedMount, XraySourceKind, XrayVfs};

impl XrayVfs {
  /// Mounts each planned source that can be opened, in plan order, behind the mounts already present.
  ///
  /// Planning is a decision about the filesystem; this is the construction of the sources it named.
  ///
  /// A source that fails to open is omitted rather than fatal, so one corrupt volume does not stop a tool from reading the
  /// rest of an installation. Each omission is recorded on the VFS through [`XrayVfs::get_skipped_mounts`] — reporting it is
  /// the caller's job, because a check enumerating a mount that never opened would otherwise present a read failure as
  /// missing content. The returned mount IDs preserve plan order.
  pub fn mount_plan(&mut self, plan: &XrayMountPlan) -> XrfResult<Vec<XrayMountId>> {
    let mut mounted: Vec<XrayMountId> = Vec::with_capacity(plan.len());

    for planned in plan.get_mounts() {
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

          self.record_skipped(XraySkippedMount {
            origin: planned.origin.clone(),
            path: planned.path.clone(),
            reason: error.to_string(),
          });
        }
      }
    }

    Ok(mounted)
  }

  /// The mounts a plan produced, in plan order, where every source it names is mounted already; `None` where one is
  /// not, which only [`Self::mount_plan`] can settle.
  ///
  /// Reads the VFS without changing it, so a caller sharing one VFS between threads learns what it already holds
  /// without shutting every other reader out. Each mount keeps the origin the plan that opened it gave.
  pub fn find_mounted_plan(&self, plan: &XrayMountPlan) -> Option<Vec<XrayMountId>> {
    plan
      .get_mounts()
      .iter()
      .map(|planned| self.planned_mount(&planned.path, planned.kind))
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
