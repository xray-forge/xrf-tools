use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_translation::{
  TranslationFile, TranslationProjectDescriptor, TranslationProjectMode, read_gamedata_in, read_source_in,
};
use xrf_utils::error_to_string;
use xrf_vfs::{XrayChangeSet, XrayInputRecord, XrayRoots, XraySearch};

use crate::core::assets::AssetMountState;
use crate::core::jobs::resolve_lease_path;
use crate::core::session::{SessionDerived, SessionId, SessionScopes, SessionSnapshot, Sessions};
use crate::core::types::TauriResult;
use crate::plugins::translations::state::{TranslationSaveOutcome, TranslationSavePlan};

/// Owns publication; domain save planning retains exactly the snapshot it read, and a commit moves each open project
/// along by reading it again.
#[derive(Clone)]
pub struct TranslationProjectState {
  /// The projects open, each held by the tool that opened it.
  pub projects: Sessions<TranslationProjectDescriptor>,
}

impl TranslationProjectState {
  /// Whatever scope `scopes` releases closes the projects it held.
  pub fn new(scopes: &SessionScopes) -> Self {
    Self {
      projects: Sessions::new("translations", scopes).with_derivation(Self::derive_project),
    }
  }

  /// Reads a translations project through `search`, recording every file and listing it reads: what an open commits,
  /// and what a commit's follow derives again.
  ///
  /// The caller's mode is obeyed, not re-derived: the two layouts save to different files.
  ///
  /// # Errors
  ///
  /// Returns an error when the prefix is not a logical path.
  pub fn read_project(
    search: XraySearch,
    roots: &XrayRoots,
    mode: TranslationProjectMode,
    prefix: &str,
  ) -> TauriResult<SessionDerived<TranslationProjectDescriptor>> {
    let inputs: XrayInputRecord = XrayInputRecord::new();
    let search: XraySearch = search.with_recording(&inputs);
    let descriptor: TranslationProjectDescriptor = match mode {
      TranslationProjectMode::Source => read_source_in(&search, roots, prefix),
      TranslationProjectMode::Gamedata => read_gamedata_in(&search, roots, prefix),
    }
    .map_err(error_to_string)?;

    Ok(SessionDerived::new(descriptor, inputs))
  }

  /// The open project read again over the roots it was opened over, once a commit moved a file it read.
  fn derive_project(
    project: &SessionSnapshot<TranslationProjectDescriptor>,
    assets: &AssetMountState,
  ) -> TauriResult<SessionDerived<TranslationProjectDescriptor>> {
    Self::read_project(
      assets.search(&project.roots)?,
      &project.roots,
      project.mode,
      &project.prefix,
    )
  }

  /// Resolves a save against the caller's document before touching any file.
  pub fn begin_save(&self, id: SessionId, file: &str) -> TauriResult<TranslationSavePlan> {
    let project: Arc<SessionSnapshot<TranslationProjectDescriptor>> = self.projects.require(id)?;
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
  /// began in is still open; stale when its tool closed it meanwhile.
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
    let Ok(project) = self.projects.require(plan.project.session_id) else {
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

  /// Refuse `directory` while any open editor session overlaps it.
  pub fn require_no_open_session_over(&self, directory: &Path) -> TauriResult<()> {
    let target: PathBuf = resolve_lease_path(directory)?;

    for root in self
      .projects
      .list_opened()?
      .iter()
      .flat_map(|project| project.roots.roots.clone())
    {
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
