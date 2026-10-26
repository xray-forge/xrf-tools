use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_translation::{
  TranslationEdit, TranslationEditor, TranslationFile, TranslationProjectDescriptor, TranslationProjectMode,
  TranslationSource,
};
use xrf_utils::error_to_string;
use xrf_vfs::{XrayChangeSet, XrayInputKind, XrayInputRecord, XrayLogicalPath, XrayRoots, XraySearch};

use crate::{TranslationProjects, TranslationSaveOutcome};

/// What one save was addressed to, taken as a single read before the filesystem work begins.
#[derive(Debug)]
pub struct TranslationSavePlan {
  pub project: Arc<SessionSnapshot<TranslationProjectDescriptor>>,
  /// The logical file being saved, keyed as the project keys it.
  pub file: String,
  pub roots: XrayRoots,
  pub prefix: String,
  pub mode: TranslationProjectMode,
  /// Language to the file holding it, for the one logical file being saved.
  pub sources: HashMap<String, TranslationSource>,
}

impl TranslationSavePlan {
  /// Writes `edits` into the project `id` holds and answers the project as the commit left it while it still is.
  ///
  /// # Errors
  ///
  /// As [`Self::begin`], [`Self::write`] and [`Self::settle`].
  pub fn save(
    projects: &TranslationProjects,
    assets: &AssetMountState,
    id: SessionId,
    file: &str,
    edits: &HashMap<String, Vec<TranslationEdit>>,
  ) -> TauriResult<TranslationSaveOutcome> {
    let plan: Self = Self::begin(projects, id, file)?;
    let changes: XrayChangeSet = plan.write(assets, edits)?;

    plan.settle(projects, &changes)
  }

  /// Resolves a save against the project `id` holds before touching any file.
  ///
  /// # Errors
  ///
  /// Returns an error when `id` holds no project, or the project holds no `file`.
  pub fn begin(projects: &TranslationProjects, id: SessionId, file: &str) -> TauriResult<Self> {
    let project: Arc<SessionSnapshot<TranslationProjectDescriptor>> = projects.require(id)?;
    let entry: &TranslationFile = project
      .files
      .get(file)
      .ok_or_else(|| format!("Translations file '{file}' is not part of the open project"))?;

    Ok(Self {
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

  /// Commits `edits` to the files the plan names as one transaction, touching no translations state: the commit moves
  /// every open session along, this project's among them, which reads itself again.
  ///
  /// # Errors
  ///
  /// Refused, with nothing written, when a file it writes changed since the plan's project read it, so a save never
  /// overwrites what it did not see; a change to any other file of the project does not stand in its way.
  pub fn write(
    &self,
    assets: &AssetMountState,
    edits: &HashMap<String, Vec<TranslationEdit>>,
  ) -> TauriResult<XrayChangeSet> {
    let sources: XraySearch = assets
      .search(&self.roots)?
      .with_optional_prefix(Some(&self.prefix))
      .map_err(error_to_string)?;
    let written: HashSet<XrayLogicalPath> = self
      .sources
      .values()
      .filter_map(|source| XrayLogicalPath::new(&source.logical_path).ok())
      .collect();
    let expected: XrayInputRecord = self
      .project
      .inputs
      .to_filtered(|input| matches!(input.get_kind(), XrayInputKind::File(path) if written.contains(path)));

    assets.commit(
      &TranslationEditor::build_save_transaction(&sources, &self.file, &self.sources, edits)
        .map_err(error_to_string)?
        .with_expected(&expected),
    )
  }

  /// How the save ended once its commit moved the session along: saved, as the project now reads, while the project it
  /// began in is still open; stale when its tool closed it meanwhile.
  ///
  /// # Errors
  ///
  /// Returns an error when the open project did not reach what the commit left, which is a project that could not be
  /// read again: the edits are on disk all the same.
  pub fn settle(&self, projects: &TranslationProjects, changes: &XrayChangeSet) -> TauriResult<TranslationSaveOutcome> {
    let Ok(project) = projects.require(self.project.session_id) else {
      return Ok(TranslationSaveOutcome::Stale);
    };

    if !changes.get_changes().is_empty() && project.revision < changes.get_revision() {
      return Err(format!(
        "Saved '{}', but the project could not be read again from what the save left",
        self.file
      ));
    }

    Ok(TranslationSaveOutcome::Saved { project })
  }
}
