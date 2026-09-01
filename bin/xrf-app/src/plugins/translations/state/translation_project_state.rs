use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use xrf_translation::{TranslationFile, TranslationProjectDescriptor};

use crate::core::jobs::to_comparable_path;
use crate::core::types::TauriResult;
use crate::plugins::translations::state::translation_save_outcome::TranslationSaveOutcome;
use crate::plugins::translations::state::translation_save_plan::TranslationSavePlan;
use crate::plugins::translations::state::translation_session::TranslationSession;

/// The open translations root.
pub struct TranslationProjectState {
  /// Private so every read and every replacement goes through a method that keeps the session beside the project.
  session: Mutex<TranslationSession>,
}

impl TranslationProjectState {
  pub fn new() -> Self {
    Self {
      session: Mutex::new(TranslationSession::new()),
    }
  }

  /// The open project as a clone, or `None` when nothing is open.
  pub fn get_project(&self) -> TauriResult<Option<TranslationProjectDescriptor>> {
    Ok(self.lock("read the translations project")?.project.clone())
  }

  /// Lend the open project for the length of one call.
  ///
  /// Lent rather than cloned because the callers that only read it - validating one value against the languages a
  /// project declared - would otherwise copy every entry of every file to answer a question about one character.
  pub fn with_project<T>(
    &self,
    consumer: impl FnOnce(&TranslationProjectDescriptor) -> TauriResult<T>,
  ) -> TauriResult<T> {
    consumer(self.lock("read the translations project")?.require_project()?)
  }

  /// Make `descriptor` the open project, under a session of its own.
  pub fn open_project(&self, descriptor: TranslationProjectDescriptor) -> TauriResult<()> {
    self.lock("open the translations project")?.restart(Some(descriptor));

    Ok(())
  }

  /// Drop the open project, under a session of its own.
  ///
  /// A close starts a new session for the same reason an open does: a save that began before it must not put back what
  /// the close removed.
  pub fn close_project(&self) -> TauriResult<()> {
    self.lock("close the translations project")?.restart(None);

    Ok(())
  }

  /// What a save of `file` is addressed to, read while the project is still held.
  pub fn begin_save(&self, file: &str) -> TauriResult<TranslationSavePlan> {
    let session: MutexGuard<TranslationSession> = self.lock("save a translations file")?;
    let descriptor: &TranslationProjectDescriptor = session.require_project()?;
    let entry: &TranslationFile = descriptor
      .files
      .get(file)
      .ok_or_else(|| format!("Translations file '{file}' is not part of the open project"))?;

    Ok(TranslationSavePlan {
      session_id: session.id,
      file: file.to_owned(),
      roots: descriptor.roots.clone(),
      prefix: descriptor.prefix.clone(),
      mode: descriptor.mode,
      sources: entry
        .sources
        .iter()
        .map(|(language, source)| (language.clone(), source.clone()))
        .collect(),
    })
  }

  /// Adopt `refreshed` as the open project, but only while `plan`'s session is still the open one.
  ///
  /// A mismatch leaves the newer state exactly as it is, session included: nothing about a save that arrived late
  /// makes it a new opening.
  pub fn commit_save(
    &self,
    plan: &TranslationSavePlan,
    refreshed: TranslationProjectDescriptor,
  ) -> TauriResult<TranslationSaveOutcome> {
    let mut session: MutexGuard<TranslationSession> = self.lock("finish saving a translations file")?;

    if session.id != plan.session_id {
      return Ok(TranslationSaveOutcome::Stale);
    }

    session.project = Some(refreshed.clone());

    Ok(TranslationSaveOutcome::Saved {
      project: Box::new(refreshed),
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
    let session: MutexGuard<TranslationSession> = self.lock("check the open translations project")?;

    let Some(project) = session.project.as_ref() else {
      return Ok(());
    };

    let target: String = to_comparable_path(directory);

    for root in &project.roots.roots {
      let open: String = to_comparable_path(&root.path);

      if is_within(&target, &open) || is_within(&open, &target) {
        return Err(format!(
          "Close the open translations project at '{}' before formatting '{}': the editor holds unsaved views of those files.",
          root.path.display(),
          directory.display()
        ));
      }
    }

    Ok(())
  }

  fn lock(&self, action: &str) -> TauriResult<MutexGuard<'_, TranslationSession>> {
    self
      .session
      .lock()
      .map_err(|error| format!("Failed to {action} - translations state is unavailable: {error}"))
  }
}

/// Whether `inner` names the same place as `outer` or something beneath it.
///
/// Compared on the comparable spelling both sides already use for leases, and on separator boundaries so `c:\gamedata`
/// does not read as containing `c:\gamedata-backup`.
fn is_within(inner: &str, outer: &str) -> bool {
  inner == outer
    || inner
      .strip_prefix(outer)
      .is_some_and(|rest| rest.starts_with('\\') || rest.starts_with('/'))
}
