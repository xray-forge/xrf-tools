use std::path::{Path, PathBuf};

use xrf_app_core::TauriResult;
use xrf_app_core::jobs::JobResource;

use crate::TranslationProjects;

/// What keeps a formatting run off the files an open editor holds: the editor's in-memory buffers are not covered by
/// any lease, and a save after the format would put the pre-format content back.
pub struct TranslationFormatGuard;

impl TranslationFormatGuard {
  /// Refuses `directory` while any open project overlaps it.
  ///
  /// # Errors
  ///
  /// Returns an error naming the open project's root, or why a path cannot be resolved.
  pub fn require_none_over(projects: &TranslationProjects, directory: &Path) -> TauriResult<()> {
    let target: PathBuf = JobResource::resolve_path(directory)?;

    for root in projects
      .list_opened()
      .iter()
      .flat_map(|project| project.roots.roots.clone())
    {
      let open: PathBuf = JobResource::resolve_path(&root.path)?;

      if target.starts_with(&open) || open.starts_with(&target) {
        return Err(format!(
          "Close the open translations project at '{}' before formatting '{}': the editor holds unsaved views of those files.",
          root.path.display(),
          directory.display()
        ));
      }
    }

    Ok(())
  }
}
