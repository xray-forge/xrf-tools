use std::sync::{Arc, Mutex, MutexGuard};

use xrf_ltx::{LtxDocumentSource, LtxEntryVerification, LtxProject};
use xrf_ltx_inspect::{LtxAnchoredFinding, LtxRootReader};
use xrf_vfs::XrayLogicalPath;

use crate::core::session::Session;
use crate::core::types::TauriResult;
use crate::plugins::configs::descriptor::ConfigsProjectDescriptor;
use crate::plugins::configs::resolved_root::ConfigsResolvedRoot;

/// One opened configs project, and whatever has been resolved out of it.
pub struct ConfigsProject {
  pub project: Arc<LtxProject>,
  /// Shared rather than owned: the inventory inside it is thousands of entries on an installation, and a restore
  /// answers with the same one the open did rather than a copy of it.
  pub descriptor: Arc<ConfigsProjectDescriptor>,
  /// The one resolved root held, and the entry point it came from.
  resolved: Mutex<Option<Arc<ConfigsResolvedRoot>>>,
}

impl ConfigsProject {
  pub fn new(project: LtxProject, descriptor: Arc<ConfigsProjectDescriptor>) -> Self {
    Self {
      descriptor,
      project: Arc::new(project),
      resolved: Mutex::new(None),
    }
  }

  /// The resolved root for one entry point, produced on the first ask and held until another is asked for.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry point cannot be read or resolved.
  pub fn resolve(&self, entry: &XrayLogicalPath, is_explaining: bool) -> TauriResult<Arc<ConfigsResolvedRoot>> {
    if let Some(held) = self.held_resolution()?.as_ref()
      && held.entry == *entry
      && (held.is_explained || !is_explaining)
    {
      return Ok(Arc::clone(held));
    }

    let resolved: Arc<ConfigsResolvedRoot> = Arc::new(if is_explaining {
      self.project.forget_root(entry);

      ConfigsResolvedRoot::explained(
        entry.clone(),
        self
          .project
          .resolve_explained(entry)
          .map_err(|error| format!("Cannot resolve '{}': {error}", entry.as_str()))?,
      )
    } else {
      ConfigsResolvedRoot::plain(
        entry.clone(),
        self
          .project
          .read_resolution(entry)
          .map_err(|error| format!("Cannot resolve '{}': {error}", entry.as_str()))?,
      )
    });

    *self.held_resolution()? = Some(Arc::clone(&resolved));

    Ok(resolved)
  }

  /// Lends a reader over one resolved root for the length of one call.
  ///
  /// # Errors
  ///
  /// Whatever resolving the entry point, or the consumer, answers with.
  pub fn with_reader<T>(
    &self,
    entry: &XrayLogicalPath,
    is_explaining: bool,
    consumer: impl FnOnce(&LtxRootReader, &ConfigsResolvedRoot) -> TauriResult<T>,
  ) -> TauriResult<T> {
    let resolved: Arc<ConfigsResolvedRoot> = self.resolve(entry, is_explaining)?;
    let source = self.project.document_source();

    let reader: LtxRootReader = LtxRootReader::new(
      resolved.entry.as_str(),
      self.project.get_dialect().get_name(),
      &resolved.resolution,
      &source as &dyn LtxDocumentSource,
    )
    .with_declared_schemes(&self.project.ltx_scheme_declarations);

    consumer(&reader, &resolved)
  }

  /// Everything wrong with one root, verified once and kept beside the resolution it is about.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry point cannot be resolved, verified, or read back.
  pub fn find_problems(&self, entry: &XrayLogicalPath) -> TauriResult<Arc<Vec<LtxAnchoredFinding>>> {
    // Findings anchor a scheme error to the file that wrote the field, which only a recorded origin can name.
    self.with_reader(entry, true, |reader, resolved| {
      if let Some(held) = resolved.get_findings()? {
        return Ok(held);
      }

      let verification: LtxEntryVerification = self
        .project
        .verify_resolved(&resolved.entry, &resolved.resolution.ltx)
        .map_err(|error| format!("Cannot verify '{}': {error}", resolved.entry.as_str()))?;

      let findings: Arc<Vec<LtxAnchoredFinding>> = Arc::new(
        reader
          .read_findings(&verification.errors)
          .map_err(|error| format!("Cannot anchor the findings of '{}': {error}", resolved.entry.as_str()))?,
      );

      resolved.hold_findings(Arc::clone(&findings))?;

      Ok(findings)
    })
  }

  fn held_resolution(&self) -> TauriResult<MutexGuard<'_, Option<Arc<ConfigsResolvedRoot>>>> {
    self
      .resolved
      .lock()
      .map_err(|error| format!("The configs resolution cache is unavailable: {error}"))
  }
}

/// The committed project and its pending replacement.
pub type ConfigsState = Session<ConfigsProject>;
