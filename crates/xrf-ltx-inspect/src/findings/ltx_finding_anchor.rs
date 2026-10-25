use xrf_error::XrfResult;
use xrf_ltx::{LtxDocument, LtxDocumentSource, LtxFieldOrigin, LtxFinding, LtxFindingKind, LtxResolution};

use crate::findings::LtxAnchoredFinding;
use crate::structure::{LtxDocumentScan, LtxFileStructure};

/// Places findings on the lines that produced them.
///
/// Reached through [`crate::LtxRootReader`]; see its `read_findings` and `read_file_findings`.
pub(crate) struct LtxFindingAnchor {}

impl LtxFindingAnchor {
  /// Every finding a verification of one root made, each placed at its line.
  ///
  /// Reached through [`crate::LtxRootReader`]; see its `read_findings`.
  pub(crate) fn read_findings(
    resolution: &LtxResolution,
    source: &dyn LtxDocumentSource,
    findings: &[LtxFinding],
  ) -> XrfResult<Vec<LtxAnchoredFinding>> {
    findings
      .iter()
      .map(|finding| {
        Ok(LtxAnchoredFinding {
          line: Self::find_finding_line(resolution, source, finding)?,
          finding: finding.clone(),
        })
      })
      .collect()
  }

  /// What is wrong with one file rather than with the root: it will not parse, or an include reached nothing.
  ///
  /// Needs no resolution, because the structure a viewer already holds carries both answers - which is what lets opening
  /// a file report them without paying for a verification it did not ask for.
  pub(crate) fn read_file_findings(entry: &str, structure: &LtxFileStructure) -> Vec<LtxAnchoredFinding> {
    let mut findings: Vec<LtxAnchoredFinding> = Vec::new();

    if let Some(parse_error) = &structure.parse_error {
      findings.push(Self::file_finding(
        LtxFindingKind::Parse,
        entry,
        structure,
        Some(parse_error.line),
        parse_error.message.clone(),
        None,
      ));
    }

    // Keys above the first header of a config that has headers. Nothing loads them: Anomaly's ini loader drops a key
    // it reads before any header, and vanilla hands the null section to `insert_item`. So what this tool resolves and
    // what the game makes of the same file differ, silently.
    //
    // Judged on the shape rather than on whether anything includes the file. A config that declares no section at all
    // is a list - a roll of names read by the console or by a script rather than loaded as an ini - and reporting that
    // would warn about `default_controls.ltx` and every `rspec_*.ltx` in both reference trees. One that declares
    // sections and then strands keys above them is the other thing entirely: in both trees, every instance of it is a
    // comment somebody wrote without a `;`.
    if !structure.root_entries.is_empty() && !structure.sections.is_empty() {
      findings.push(Self::file_finding(
        LtxFindingKind::RootKeys,
        entry,
        structure,
        // The first of them, because that is where the run starts and where a reader has to look.
        structure.root_entries.first().map(|first| first.line),
        format!(
          "{} key(s) written above [{}], where no section holds them",
          structure.root_entries.len(),
          structure.sections.first().map_or("", |section| section.name.as_str())
        ),
        Some(String::from(
          "Anomaly's ini loader drops keys read before the first section header; vanilla X-Ray dereferences a null section",
        )),
      ));
    }

    for include in &structure.includes {
      if !include.resolved.is_empty() {
        continue;
      }

      findings.push(Self::file_finding(
        LtxFindingKind::Include,
        entry,
        structure,
        Some(include.line),
        format!("Include '{}' resolved to no file", include.statement),
        None,
      ));
    }

    findings
  }

  /// A finding about one file as a whole, at the line its structure already knows.
  fn file_finding(
    kind: LtxFindingKind,
    entry: &str,
    structure: &LtxFileStructure,
    line: Option<u32>,
    message: String,
    engine_behaviour: Option<String>,
  ) -> LtxAnchoredFinding {
    LtxAnchoredFinding {
      finding: LtxFinding {
        engine_behaviour,
        entry: String::from(entry),
        field: None,
        file: Some(structure.path.clone()),
        kind,
        message,
        section: None,
      },
      line,
    }
  }

  /// The line one finding marks in the config it names: a scheme finding's field where the section writes it, any other
  /// finding about a section its header.
  fn find_finding_line(
    resolution: &LtxResolution,
    source: &dyn LtxDocumentSource,
    finding: &LtxFinding,
  ) -> XrfResult<Option<u32>> {
    let (Some(file), Some(section)) = (&finding.file, &finding.section) else {
      return Ok(None);
    };

    let Some(document) = source.read_document(file)? else {
      return Ok(None);
    };

    Ok(match finding.kind {
      LtxFindingKind::Scheme => Self::find_line(resolution, &document, section, finding.field.as_deref()),
      _ => LtxDocumentScan::find_section_header_line(&document, section),
    })
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
}
