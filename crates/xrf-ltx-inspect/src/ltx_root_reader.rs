use std::cell::RefCell;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxDocumentSource, LtxResolution};

use crate::findings::{LtxAnchoredFinding, LtxFindingAnchor};
use crate::resolved::{LtxResolvedIndex, LtxResolvedReader, LtxResolvedSection};
use crate::structure::{LtxDeclaredParents, LtxFileStructure, LtxStructureReader};

/// Everything one resolved root can be asked, and the world its configs came from.
///
/// One reader rather than one per question, because every question below is about the same root and needs the same
/// three things to answer: which entry point was resolved, what it resolved to, and where its configs live. Splitting
/// them left a caller building three near-identical contexts and remembering which subset each wanted; a surface
/// showing a file, its findings and the sections it resolves to is one screen, not three.
///
/// Reading is cheap after the first answer. The documents behind a root are already parsed and retained by whatever
/// produced the resolution, and the headers this reader has to look back at are kept for the
/// reader's lifetime rather than per question.
///
/// Borrows everything it reads: a root's resolution is large and belongs to whoever decided to keep it, so this is a
/// lens over that decision rather than a second owner of it.
pub struct LtxRootReader<'a> {
  entry: &'a str,
  dialect: &'a str,
  resolution: &'a LtxResolution,
  source: &'a dyn LtxDocumentSource,
  declared_schemes: &'a [&'a str],
  /// Headers read back from declaring configs, shared by every question this reader answers.
  ///
  /// Interior mutability so reading stays `&self`: a caller holding one reader asks several questions in no fixed
  /// order, and none of them changes what the root resolved to.
  declared_parents: RefCell<LtxDeclaredParents>,
}

impl<'a> LtxRootReader<'a> {
  /// A reader over one resolved root.
  ///
  /// `dialect` is how the dialect that produced `resolution` names itself; the resolution does not carry it, and the
  /// caller that chose the dialect is the one place that knows.
  pub fn new(
    entry: &'a str,
    dialect: &'a str,
    resolution: &'a LtxResolution,
    source: &'a dyn LtxDocumentSource,
  ) -> Self {
    Self {
      declared_parents: RefCell::new(LtxDeclaredParents::default()),
      declared_schemes: &[],
      dialect,
      entry,
      resolution,
      source,
    }
  }

  /// The scheme names the project declares, so a binding can be reported as bound or dangling.
  ///
  /// Left empty by default rather than defaulted to "declared": a reader given nothing says every binding is
  /// undeclared, which is what a project with no scheme files actually means.
  pub fn with_declared_schemes(mut self, declared_schemes: &'a [&'a str]) -> Self {
    self.declared_schemes = declared_schemes;

    self
  }

  /// One config as written, judged against this root.
  ///
  /// Whether a parent resolves and which scheme a section ends up bound to are questions about the whole root, so a
  /// file read on its own could answer neither. `entry_points` is every entry point that reaches this config, which
  /// the project knows and a root does not.
  ///
  /// A file that will not parse is answered rather than refused: the viewer still opens it, shows its text and marks
  /// the line. Anything else that goes wrong is a failure of the read.
  ///
  /// # Errors
  ///
  /// Returns an error when the config is not in scope, or when resolving one of its includes fails.
  pub fn read_structure(&self, path: &str, entry_points: &[String]) -> XrfResult<LtxFileStructure> {
    LtxStructureReader::read_structure(self.resolution, self.source, self.declared_schemes, path, entry_points)
  }

  /// Every section of the root, named and counted.
  ///
  /// # Errors
  ///
  /// Returns an error when a declaring config cannot be read back.
  pub fn read_index(&self) -> XrfResult<LtxResolvedIndex> {
    LtxResolvedReader::read_index(
      self.entry,
      self.dialect,
      self.resolution,
      self.source,
      &mut self.declared_parents.borrow_mut(),
    )
  }

  /// The bodies of the named sections, in the order asked for.
  ///
  /// Addressed by name rather than by offset, so a filter applied on one side never has to be mirrored on the other
  /// and a page is always whole sections. A name the root does not hold is skipped rather than refused: a page request
  /// races an index the caller may have fetched before a reopen.
  ///
  /// # Errors
  ///
  /// Returns an error when a declaring config cannot be read back.
  pub fn read_sections(&self, names: &[&str]) -> XrfResult<Vec<LtxResolvedSection>> {
    LtxResolvedReader::read_sections(
      self.entry,
      self.resolution,
      self.source,
      &mut self.declared_parents.borrow_mut(),
      names,
    )
  }

  /// Everything wrong with the root: what verifying it found, and what the dialect said while resolving it.
  ///
  /// `errors` is what a per-entry verification answered. Both halves are anchored the same way because a reader does
  /// not care which pass noticed.
  ///
  /// # Errors
  ///
  /// Returns an error when a declaring config cannot be read back.
  pub fn read_findings(&self, errors: &[XrfError]) -> XrfResult<Vec<LtxAnchoredFinding>> {
    LtxFindingAnchor::read_findings(self.entry, self.resolution, self.source, errors)
  }

  /// What is wrong with one file rather than with the root: it will not parse, or an include reached nothing.
  ///
  /// Answered from the structure a viewer already holds, so opening a file does not pay for a verification it did not
  /// ask for.
  pub fn read_file_findings(&self, structure: &LtxFileStructure) -> Vec<LtxAnchoredFinding> {
    LtxFindingAnchor::read_file_findings(self.entry, structure)
  }
}
