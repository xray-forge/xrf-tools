use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LTX_SYMBOL_ANY, LtxDocument, LtxDocumentSource, LtxFieldOrigin, LtxResolution, LtxResolutionDiagnostic};

use crate::findings::{LtxAnchoredFinding, LtxFindingKind};
use crate::structure::{LtxDocumentScan, LtxFileStructure};

/// The config a scheme finding names and the line inside it, either of which may be unknown.
///
/// A pair rather than a record: it exists between one private helper and its one caller, and naming two fields would
/// add a type to navigate without hiding anything from either of them.
type SchemeAnchor = (Option<String>, Option<u32>);

/// Places findings on the lines that produced them.
///
/// Reached through [`crate::LtxRootReader`]; see its `read_findings` and `read_file_findings`.
pub(crate) struct LtxFindingAnchor {}

impl LtxFindingAnchor {
  /// Everything wrong with one root: what verifying it found, and what the dialect said while resolving it.
  ///
  /// Reached through [`crate::LtxRootReader`]; see its `read_findings`.
  pub(crate) fn read_findings(
    entry: &str,
    resolution: &LtxResolution,
    source: &dyn LtxDocumentSource,
    errors: &[XrfError],
  ) -> XrfResult<Vec<LtxAnchoredFinding>> {
    let mut findings: Vec<LtxAnchoredFinding> = Vec::with_capacity(errors.len() + resolution.diagnostics.len());

    for error in errors {
      findings.push(Self::anchor_error(entry, resolution, source, error)?);
    }

    for diagnostic in &resolution.diagnostics {
      findings.push(Self::anchor_diagnostic(entry, resolution, source, diagnostic)?);
    }

    Ok(findings)
  }

  /// What is wrong with one file rather than with the root: it will not parse, or an include reached nothing.
  ///
  /// Needs no resolution, because the structure a viewer already holds carries both answers - which is what lets opening
  /// a file report them without paying for a verification it did not ask for.
  pub(crate) fn read_file_findings(entry: &str, structure: &LtxFileStructure) -> Vec<LtxAnchoredFinding> {
    let mut findings: Vec<LtxAnchoredFinding> = Vec::new();

    if let Some(parse_error) = &structure.parse_error {
      findings.push(LtxAnchoredFinding {
        engine_behaviour: None,
        entry: String::from(entry),
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
        entry: String::from(entry),
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
  fn anchor_error(
    entry: &str,
    resolution: &LtxResolution,
    source: &dyn LtxDocumentSource,
    error: &XrfError,
  ) -> XrfResult<LtxAnchoredFinding> {
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
        let (file, line) = Self::anchor_scheme(resolution, source, section, field)?;

        Ok(LtxAnchoredFinding {
          engine_behaviour: None,
          entry: String::from(entry),
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
        entry: String::from(entry),
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
        entry: String::from(entry),
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
  fn anchor_diagnostic(
    entry: &str,
    resolution: &LtxResolution,
    source: &dyn LtxDocumentSource,
    diagnostic: &LtxResolutionDiagnostic,
  ) -> XrfResult<LtxAnchoredFinding> {
    // The file the dialect blamed, which for a patch is the patch rather than the base; failing that, the config that
    // declared the section.
    let file: Option<String> = diagnostic
      .file
      .clone()
      .or_else(|| Self::declaring_file(resolution, &diagnostic.section));

    let line: Option<u32> = match &file {
      Some(file) => source
        .read_document(file)?
        .and_then(|document| LtxDocumentScan::find_section_header_line(&document, &diagnostic.section)),
      None => None,
    };

    Ok(LtxAnchoredFinding {
      engine_behaviour: diagnostic.engine_behaviour.clone(),
      entry: String::from(entry),
      field: None,
      file,
      kind: LtxFindingKind::Dialect,
      line,
      message: diagnostic.message.clone(),
      section: Some(diagnostic.section.clone()),
    })
  }

  /// Where a scheme finding about one section, and possibly one field, belongs.
  fn anchor_scheme(
    resolution: &LtxResolution,
    source: &dyn LtxDocumentSource,
    section: &str,
    field: Option<&str>,
  ) -> XrfResult<SchemeAnchor> {
    let Some(file) = Self::declaring_file(resolution, section) else {
      return Ok((None, None));
    };

    let Some(document) = source.read_document(&file)? else {
      return Ok((Some(file), None));
    };

    Ok((Some(file), Self::find_line(resolution, &document, section, field)))
  }

  /// The line inside one declaring config a scheme finding marks.
  fn find_line(resolution: &LtxResolution, document: &LtxDocument, section: &str, field: Option<&str>) -> Option<u32> {
    let header: Option<u32> = LtxDocumentScan::find_section_header_line(document, section);

    // A field the section did not write itself has no line here: an inherited value is written in an ancestor, and a
    // required one that is missing was never written anywhere. Both belong on the header.
    let Some(field) = field.filter(|field| !Self::is_inherited(resolution, section, field)) else {
      return header;
    };

    LtxDocumentScan::find_section_key_line(document, section, field).or(header)
  }

  /// Whether the resolution says one field arrived by inheritance.
  fn is_inherited(resolution: &LtxResolution, section: &str, field: &str) -> bool {
    resolution
      .get_origin(section, field)
      .is_some_and(LtxFieldOrigin::is_inherited)
  }

  /// The config whose header declared one section, as the resolution stamped it.
  fn declaring_file(resolution: &LtxResolution, section: &str) -> Option<String> {
    resolution.ltx.section(section)?.get_origin().map(String::from)
  }
}
