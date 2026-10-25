use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_app_core::TauriResult;
use xrf_ltx::{LtxDocumentSource, LtxEntryVerification, LtxProject};
use xrf_ltx_inspect::{
  LtxAnchoredFinding, LtxFileStructure, LtxFileText, LtxResolvedIndex, LtxRootReader, LtxTextReader,
};
use xrf_utils::{decode_w1251_bytes_to_string, error_to_string};
use xrf_vfs::XrayLogicalPath;

use crate::{ConfigsDocument, ConfigsProjectDescriptor, ConfigsResolvedRoot};

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
  /// The project `descriptor` describes, nothing resolved yet.
  pub fn new(project: LtxProject, descriptor: Arc<ConfigsProjectDescriptor>) -> Self {
    Self {
      descriptor,
      project: Arc::new(project),
      resolved: Mutex::new(None),
    }
  }

  /// What the page lists the project by.
  pub fn describe(&self) -> Arc<ConfigsProjectDescriptor> {
    Arc::clone(&self.descriptor)
  }

  /// The resolved root for one entry point, produced on the first ask and held until another is asked for.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry point cannot be read or resolved.
  pub fn resolve(&self, entry: &XrayLogicalPath, is_explaining: bool) -> TauriResult<Arc<ConfigsResolvedRoot>> {
    if let Some(held) = self.held_resolution().as_ref()
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

    *self.held_resolution() = Some(Arc::clone(&resolved));

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

  /// Every section one entry point resolves to, named and counted, indexed once and kept beside its resolution.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry point cannot be resolved or indexed.
  pub fn index(&self, entry: &XrayLogicalPath) -> TauriResult<Arc<LtxResolvedIndex>> {
    self.with_reader(entry, true, |reader, resolved| {
      if let Some(index) = resolved.get_index() {
        return Ok(index);
      }

      let index: LtxResolvedIndex = reader
        .read_index()
        .map_err(|error| format!("Cannot index '{}': {error}", resolved.entry.as_str()))?;

      Ok(resolved.hold_index(Arc::new(index)))
    })
  }

  /// Everything wrong with one root, verified once and kept beside the resolution it is about.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry point cannot be resolved or read back.
  pub fn find_problems(&self, entry: &XrayLogicalPath) -> TauriResult<Arc<Vec<LtxAnchoredFinding>>> {
    // Findings anchor a scheme error to the file that wrote the field, which only a recorded origin can name.
    self.with_reader(entry, true, |reader, resolved| {
      if let Some(held) = resolved.get_findings() {
        return Ok(held);
      }

      let verification: LtxEntryVerification = self.project.verify_resolved(&resolved.entry, &resolved.resolution);

      let findings: Vec<LtxAnchoredFinding> = reader
        .read_findings(&verification.findings)
        .map_err(|error| format!("Cannot anchor the findings of '{}': {error}", resolved.entry.as_str()))?;

      Ok(resolved.hold_findings(Arc::new(findings)))
    })
  }

  /// One config as the authored view renders it: the text as authored, and the structure judged against its entry
  /// point's resolution.
  ///
  /// # Errors
  ///
  /// Returns an error when the config cannot be read, decoded, or read against its entry point.
  pub fn read_document(&self, path: &str) -> TauriResult<ConfigsDocument> {
    let logical: XrayLogicalPath = XrayLogicalPath::new(path).map_err(error_to_string)?;

    let bytes: Vec<u8> = self
      .project
      .search()
      .read(logical.as_str())
      .map_err(|error| format!("Cannot read '{path}': {error}"))?;

    // Configs are strict Windows-1251, loose and archived alike; a lossy decode would put a viewer and a writer at odds
    // about what the file says.
    let contents: String =
      decode_w1251_bytes_to_string(&bytes).map_err(|error| format!("Cannot decode '{path}': {error}"))?;

    let text: LtxFileText = LtxTextReader::read(logical.as_str(), &contents);

    // An entry point answers itself, so this is the file's own resolution when nothing includes it. A config reached by
    // two entry points takes the first in project order; the frontend is told which through the structure it gets back.
    let entry_points: Vec<String> = self.descriptor.inventory.list_entry_points_of(logical.as_str());

    let Some(first) = entry_points.first() else {
      // A config the project holds but no entry point reaches: an attachment under a dialect that has them. Its text is
      // still worth showing, and there is no resolution it participates in to judge it against.
      return Ok(ConfigsDocument {
        findings: Vec::new(),
        structure: LtxFileStructure::new_unreached(logical.as_str()),
        text,
      });
    };

    let entry: XrayLogicalPath = XrayLogicalPath::new(first).map_err(error_to_string)?;

    let (structure, findings): (LtxFileStructure, Vec<LtxAnchoredFinding>) =
      // No origins: a structure read asks whether a parent resolves and what `$scheme` a section ends with, and a
      // plain resolution answers both for half the time and half the memory of one that records every field's origin.
      self.with_reader(&entry, false, |reader, _| {
        let structure: LtxFileStructure = reader
          .read_structure(logical.as_str(), &entry_points)
          .map_err(|error| format!("Cannot read the structure of '{path}': {error}"))?;
        let findings: Vec<LtxAnchoredFinding> = reader.read_file_findings(&structure);

        Ok((structure, findings))
      })?;

    Ok(ConfigsDocument {
      findings,
      structure,
      text,
    })
  }

  /// The held resolution, whatever a panicking reader left: it is only ever replaced whole.
  fn held_resolution(&self) -> MutexGuard<'_, Option<Arc<ConfigsResolvedRoot>>> {
    self.resolved.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
