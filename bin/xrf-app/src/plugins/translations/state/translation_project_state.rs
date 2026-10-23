use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_translation::{TranslationFile, TranslationProjectDescriptor};
use xrf_vfs::XrayChangeSet;

use crate::core::jobs::resolve_lease_path;
use crate::core::session::{Session, SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::translations::project_read::derive_project;
use crate::plugins::translations::state::{TranslationSaveOutcome, TranslationSavePlan};

/// Owns publication; domain save planning retains exactly the snapshot it read, and a commit moves the open project
/// along by reading it again.
pub struct TranslationProjectState {
  /// Publication itself, in the one vocabulary every plugin addresses an opening by.
  pub session: Session<TranslationProjectDescriptor>,
}

impl TranslationProjectState {
  pub fn new() -> Self {
    Self {
      session: Session::new("translations").with_derivation(derive_project),
    }
  }

  /// Resolves a save against the caller's document before touching any file.
  pub fn begin_save(&self, id: SessionId, file: &str) -> TauriResult<TranslationSavePlan> {
    let project: Arc<SessionSnapshot<TranslationProjectDescriptor>> = self.session.require(id)?;
    let entry: &TranslationFile = project
      .files
      .get(file)
      .ok_or_else(|| format!("Translations file '{file}' is not part of the open project"))?;

    Ok(TranslationSavePlan {
      file: file.to_owned(),
      roots: project.roots.clone(),
      prefix: project.prefix.clone(),
      mode: project.mode,
      sources: entry
        .sources
        .iter()
        .map(|(language, source)| (language.clone(), source.clone()))
        .collect(),
      project,
    })
  }

  /// How a save ended once its commit moved the session along: saved, as the project now reads, while the project it
  /// began in is still the open one; stale when another open or a close replaced it meanwhile.
  ///
  /// # Errors
  ///
  /// Returns an error when the open project did not reach what the commit left, which is a project that could not be
  /// read again: the edits are on disk all the same.
  pub fn settle_save(
    &self,
    plan: &TranslationSavePlan,
    changes: &XrayChangeSet,
  ) -> TauriResult<TranslationSaveOutcome> {
    let Some(project) = self
      .session
      .get()?
      .filter(|project| project.session_id == plan.project.session_id)
    else {
      return Ok(TranslationSaveOutcome::Stale);
    };

    if !changes.get_changes().is_empty() && project.revision < changes.get_revision() {
      return Err(format!(
        "Saved '{}', but the project could not be read again from what the save left",
        plan.file
      ));
    }

    Ok(TranslationSaveOutcome::Saved { project })
  }

  /// Refuse `directory` while an open editor session overlaps it.
  pub fn require_no_open_session_over(&self, directory: &Path) -> TauriResult<()> {
    let Some(project) = self.session.get()? else {
      return Ok(());
    };

    let target: PathBuf = resolve_lease_path(directory)?;

    for root in &project.roots.roots {
      let open: PathBuf = resolve_lease_path(&root.path)?;

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
