//! Where each kind of finding lands, which is the whole reason this crate exists between the core and a viewer.

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::LtxResolution;

use crate::findings::{LtxAnchoredFinding, LtxFindingAnchor, LtxFindingKind};
use crate::structure::{LtxFileStructure, LtxStructureReader};
use crate::tests::ltx_map_source::LtxMapSource;

/// One config declaring a parent and a child, laid out so every line number in these tests is readable here.
///
/// ```text
/// 1  ; weapons
/// 2  [wpn_base]
/// 3  $scheme = weapon
/// 4  cost = 100
/// 5
/// 6  [wpn_child]:wpn_base
/// 7  description = child
/// ```
const WEAPONS: &str =
  "; weapons\n[wpn_base]\n$scheme = weapon\ncost = 100\n\n[wpn_child]:wpn_base\ndescription = child\n";

/// A scheme finding as the per-entry verification raises one.
fn scheme_error(section: &str, field: &str) -> XrfError {
  XrfError::new_scheme_error_resolved(section, field, "Invalid value", Some("system.ltx"), "system.ltx")
}

#[test]
fn a_finding_about_a_field_the_section_writes_anchors_to_that_line() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", WEAPONS)]);
  let resolution: LtxResolution = source.resolve("system.ltx")?;

  let findings: Vec<LtxAnchoredFinding> =
    LtxFindingAnchor::new("system.ltx", &resolution, &source).read_findings(&[scheme_error("wpn_base", "cost")])?;

  assert_eq!(findings[0].kind, LtxFindingKind::Scheme);
  assert_eq!(findings[0].file.as_deref(), Some("system.ltx"));
  assert_eq!(findings[0].line, Some(4));
  assert_eq!(findings[0].section.as_deref(), Some("wpn_base"));
  assert_eq!(findings[0].field.as_deref(), Some("cost"));

  Ok(())
}

#[test]
fn a_finding_with_no_field_anchors_to_the_header() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", WEAPONS)]);
  let resolution: LtxResolution = source.resolve("system.ltx")?;

  let findings: Vec<LtxAnchoredFinding> =
    LtxFindingAnchor::new("system.ltx", &resolution, &source).read_findings(&[scheme_error("wpn_base", "*")])?;

  assert_eq!(findings[0].line, Some(2));
  assert_eq!(findings[0].field, None, "`*` is the absence of a field, not a field");

  Ok(())
}

#[test]
fn a_finding_about_an_inherited_field_anchors_to_the_header_that_pulled_it_in() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", WEAPONS)]);
  let resolution: LtxResolution = source.resolve("system.ltx")?;

  let findings: Vec<LtxAnchoredFinding> =
    LtxFindingAnchor::new("system.ltx", &resolution, &source).read_findings(&[scheme_error("wpn_child", "cost")])?;

  assert_eq!(
    findings[0].line,
    Some(6),
    "wpn_child writes no `cost` line; the header is what brought the value in"
  );
  assert_eq!(findings[0].field.as_deref(), Some("cost"));

  Ok(())
}

#[test]
fn a_finding_about_a_field_the_child_writes_anchors_inside_the_child() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", WEAPONS)]);
  let resolution: LtxResolution = source.resolve("system.ltx")?;

  let findings: Vec<LtxAnchoredFinding> = LtxFindingAnchor::new("system.ltx", &resolution, &source)
    .read_findings(&[scheme_error("wpn_child", "description")])?;

  assert_eq!(findings[0].line, Some(7));

  Ok(())
}

#[test]
fn a_finding_anchors_in_the_config_that_declares_the_section_not_the_entry_point() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[
    ("system.ltx", "#include \"items\\w_base.ltx\"\n"),
    ("items\\w_base.ltx", "[wpn_base]\ncost = 100\n"),
  ]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let findings: Vec<LtxAnchoredFinding> =
    LtxFindingAnchor::new("system.ltx", &resolution, &source).read_findings(&[scheme_error("wpn_base", "cost")])?;

  assert_eq!(findings[0].file.as_deref(), Some("items\\w_base.ltx"));
  assert_eq!(findings[0].line, Some(2));
  assert_eq!(
    findings[0].entry, "system.ltx",
    "still keyed by the root it was found under"
  );

  Ok(())
}

#[test]
fn a_parse_error_carries_its_own_line() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[
    ("system.ltx", "[wpn_base]\ncost = 100\n"),
    ("broken.ltx", "[wpn_base]\ncost = 100\n[unclosed\n"),
  ]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let structure: LtxFileStructure = LtxStructureReader::new(&resolution, &source).read("broken.ltx", &[])?;

  let findings: Vec<LtxAnchoredFinding> =
    LtxFindingAnchor::new("system.ltx", &resolution, &source).read_file_findings(&structure);

  assert_eq!(findings[0].kind, LtxFindingKind::Parse);
  assert_eq!(findings[0].file.as_deref(), Some("broken.ltx"));
  // The parser's own position, carried through untouched - nothing here recomputes where it stopped.
  assert_eq!(findings[0].line, Some(4));

  Ok(())
}

#[test]
fn an_include_that_reached_nothing_is_a_finding_on_its_statement() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", "[wpn_base]\ncost = 100\n#include \"absent.ltx\"\n")]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let structure: LtxFileStructure = LtxStructureReader::new(&resolution, &source).read("system.ltx", &[])?;

  let findings: Vec<LtxAnchoredFinding> =
    LtxFindingAnchor::new("system.ltx", &resolution, &source).read_file_findings(&structure);

  assert_eq!(findings.len(), 1);
  assert_eq!(findings[0].kind, LtxFindingKind::Include);
  assert_eq!(findings[0].line, Some(3));

  Ok(())
}
