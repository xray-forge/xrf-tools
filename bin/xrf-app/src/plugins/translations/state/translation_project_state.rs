use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_translation::{TranslationFile, TranslationProjectDescriptor};

use crate::core::jobs::resolve_lease_path;
use crate::core::session::{DocumentSession, DocumentSessionId, DocumentSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::translations::state::{TranslationSaveOutcome, TranslationSavePlan};

/// Owns publication; domain save planning retains exactly the snapshot it read.
pub struct TranslationProjectState {
  session: DocumentSession<TranslationProjectDescriptor>,
}

impl TranslationProjectState {
  pub fn new() -> Self {
    Self {
      session: DocumentSession::new("translations"),
    }
  }

  pub fn get_project(&self) -> TauriResult<Option<Arc<DocumentSnapshot<TranslationProjectDescriptor>>>> {
    self.session.get()
  }

  pub fn require(&self, id: DocumentSessionId) -> TauriResult<Arc<DocumentSnapshot<TranslationProjectDescriptor>>> {
    self.session.require(id)
  }

  pub fn begin_open(&self, id: DocumentSessionId) -> TauriResult<()> {
    self.session.begin_open(id)
  }

  pub fn open_project(
    &self,
    id: DocumentSessionId,
    descriptor: TranslationProjectDescriptor,
  ) -> TauriResult<Arc<DocumentSnapshot<TranslationProjectDescriptor>>> {
    self.session.commit_open(id, descriptor)
  }

  pub fn close_project(&self, ids: &[DocumentSessionId]) -> TauriResult<()> {
    self.session.close(ids)
  }

  /// Resolves a save against the caller's document before touching any file.
  pub fn begin_save(&self, id: DocumentSessionId, file: &str) -> TauriResult<TranslationSavePlan> {
    let project: Arc<DocumentSnapshot<TranslationProjectDescriptor>> = self.session.require(id)?;
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

  /// A completed write may report success only for the snapshot it refreshed.
  pub fn commit_save(
    &self,
    plan: &TranslationSavePlan,
    refreshed: TranslationProjectDescriptor,
  ) -> TauriResult<TranslationSaveOutcome> {
    Ok(match self.session.replace(&plan.project, refreshed)? {
      Some(project) => TranslationSaveOutcome::Saved { project },
      None => TranslationSaveOutcome::Stale,
    })
  }

  /// Refuse `directory` while an open editor session overlaps it.
  ///
  /// The editor holds buffers in memory, and a lease does not cover them because a session is not a job. Rewriting the
  /// files under an open project would leave those buffers stale, and the next `save_file` would put the pre-format
  /// content back — undoing the formatting without anybody being told. Closing the project is one click, so refusing
  /// is cheap and losing a translator's view of a file is not.
  ///
  /// Overlap is containment either way: formatting a parent of the open root reaches its files, and formatting a
  /// subtree of it reaches some of them.
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
