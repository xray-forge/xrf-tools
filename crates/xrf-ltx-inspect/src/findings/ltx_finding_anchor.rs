use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LTX_SYMBOL_ANY, LtxDocument, LtxDocumentSource, LtxFieldOrigin, LtxResolution, LtxResolutionDiagnostic};

use crate::findings::{LtxAnchoredFinding, LtxFindingKind};
use crate::structure::{LtxFileStructure, find_section_header_line, find_section_key_line};

/// Where a finding is, and where a finding is anchored.
type Anchor = (Option<String>, Option<u32>);

/// Places one root's findings on the lines that produced them.
pub struct LtxFindingAnchor<'a> {
  entry: &'a str,
  resolution: &'a LtxResolution,
  source: &'a dyn LtxDocumentSource,
}

impl<'a> LtxFindingAnchor<'a> {
  /// An anchor over one resolved root and the world its configs came from.
  pub fn new(entry: &'a str, resolution: &'a LtxResolution, source: &'a dyn LtxDocumentSource) -> Self {
    Self {
      entry,
      resolution,
      source,
    }
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
    let mut findings: Vec<LtxAnchoredFinding> = Vec::with_capacity(errors.len() + self.resolution.diagnostics.len());

    for error in errors {
      findings.push(self.anchor_error(error)?);
    }

    for diagnostic in &self.resolution.diagnostics {
      findings.push(self.anchor_diagnostic(diagnostic)?);
    }

    Ok(findings)
  }

  /// What is wrong with one file rather than with the root: it will not parse, or an include reached nothing.
  ///
  /// Separate from [`Self::read_findings`] because neither needs the resolution and both are answered by the structure
  /// the viewer already holds, so opening a file does not pay for a verification it did not ask for.
  pub fn read_file_findings(&self, structure: &LtxFileStructure) -> Vec<LtxAnchoredFinding> {
    let mut findings: Vec<LtxAnchoredFinding> = Vec::new();

    if let Some(parse_error) = &structure.parse_error {
      findings.push(LtxAnchoredFinding {
        engine_behaviour: None,
        entry: String::from(self.entry),
        field: None,
        file: Some(structure.path.clone()),
        kind: LtxFindingKind::Parse,
        line: Some(parse_error.line),
        message: parse_error.message.clone(),
        section: None,
      });
    }

    for include in &structure.includes {
      if !include.resolved.is_empty() {
        continue;
      }

      findings.push(LtxAnchoredFinding {
        engine_behaviour: None,
        entry: String::from(self.entry),
        field: None,
        file: Some(structure.path.clone()),
        kind: LtxFindingKind::Include,
        line: Some(include.line),
        message: format!("Include '{}' resolved to no file", include.statement),
        section: None,
      });
    }

    findings
  }

  /// One verification error, placed at the statement it is about.
  fn anchor_error(&self, error: &XrfError) -> XrfResult<LtxAnchoredFinding> {
    match error {
      XrfError::LtxScheme {
        section,
        field,
        message,
        ..
      } => {
        // `*` is how a scheme finding says it is about the section rather than about a field, which is what a missing
        // scheme declaration is.
        let field: Option<&str> = (field != LTX_SYMBOL_ANY).then_some(field.as_str());
        let (file, line) = self.anchor_scheme(section, field)?;

        Ok(LtxAnchoredFinding {
          engine_behaviour: None,
          entry: String::from(self.entry),
          field: field.map(String::from),
          file,
          kind: LtxFindingKind::Scheme,
          line,
          message: message.clone(),
          section: Some(section.clone()),
        })
      }
      XrfError::LtxParse { line, message, .. } => Ok(LtxAnchoredFinding {
        engine_behaviour: None,
        entry: String::from(self.entry),
        field: None,
        // The parser reports where it stopped, not which file it was reading; whatever caught the error knows that.
        file: None,
        kind: LtxFindingKind::Parse,
        line: Some(*line as u32),
        message: message.clone(),
        section: None,
      }),
      // Anything else reached this pass without a statement to point at, and saying so is better than guessing one.
      error => Ok(LtxAnchoredFinding {
        engine_behaviour: None,
        entry: String::from(self.entry),
        field: None,
        file: None,
        kind: LtxFindingKind::Dialect,
        line: None,
        message: error.to_string(),
        section: None,
      }),
    }
  }

  /// One dialect diagnostic, placed at the header of the section it is about.
  fn anchor_diagnostic(&self, diagnostic: &LtxResolutionDiagnostic) -> XrfResult<LtxAnchoredFinding> {
    // The file the dialect blamed, which for a patch is the patch rather than the base; failing that, the config that
    // declared the section.
    let file: Option<String> = diagnostic
      .file
      .clone()
      .or_else(|| self.declaring_file(&diagnostic.section));

    let line: Option<u32> = match &file {
      Some(file) => self
        .source
        .read_document(file)?
        .and_then(|document| find_section_header_line(&document, &diagnostic.section)),
      None => None,
    };

    Ok(LtxAnchoredFinding {
      engine_behaviour: diagnostic.engine_behaviour.clone(),
      entry: String::from(self.entry),
      field: None,
      file,
      kind: LtxFindingKind::Dialect,
      line,
      message: diagnostic.message.clone(),
      section: Some(diagnostic.section.clone()),
    })
  }

  /// Where a scheme finding about one section, and possibly one field, belongs.
  fn anchor_scheme(&self, section: &str, field: Option<&str>) -> XrfResult<Anchor> {
    let Some(file) = self.declaring_file(section) else {
      return Ok((None, None));
    };

    let Some(document) = self.source.read_document(&file)? else {
      return Ok((Some(file), None));
    };

    Ok((Some(file), self.find_line(&document, section, field)))
  }

  /// The line inside one declaring config a scheme finding marks.
  fn find_line(&self, document: &LtxDocument, section: &str, field: Option<&str>) -> Option<u32> {
    let header: Option<u32> = find_section_header_line(document, section);

    // A field the section did not write itself has no line here: an inherited value is written in an ancestor, and a
    // required one that is missing was never written anywhere. Both belong on the header.
    let Some(field) = field.filter(|field| !self.is_inherited(section, field)) else {
      return header;
    };

    find_section_key_line(document, section, field).or(header)
  }

  /// Whether the resolution says one field arrived by inheritance.
  fn is_inherited(&self, section: &str, field: &str) -> bool {
    self
      .resolution
      .get_origin(section, field)
      .is_some_and(LtxFieldOrigin::is_inherited)
  }

  /// The config whose header declared one section, as the resolution stamped it.
  fn declaring_file(&self, section: &str) -> Option<String> {
    self.resolution.ltx.section(section)?.get_origin().map(String::from)
  }
}
