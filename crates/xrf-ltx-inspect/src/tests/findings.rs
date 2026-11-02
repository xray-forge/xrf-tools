//! Where each kind of finding lands, which is the whole reason this crate exists between the core and a viewer.

use xrf_error::XrfResult;
use xrf_ltx::{LtxFinding, LtxFindingKind, LtxResolution, LtxResolutionDiagnostic};

use crate::findings::LtxAnchoredFinding;
use crate::ltx_root_reader::LtxRootReader;
use crate::structure::LtxFileStructure;
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

/// A scheme finding as the per-entry verification raises one: at the config that declared the section.
fn scheme_finding(resolution: &LtxResolution, section: &str, field: Option<&str>) -> LtxFinding {
  let declared_in: Option<String> = resolution
    .ltx
    .section(section)
    .and_then(|it| it.get_origin())
    .map(String::from);

  LtxFinding::new_scheme("system.ltx", declared_in, section, field, "Invalid value")
}

#[test]
fn a_finding_about_a_field_the_section_writes_anchors_to_that_line() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", WEAPONS)]);
  let resolution: LtxResolution = source.resolve("system.ltx")?;

  let findings: Vec<LtxAnchoredFinding> = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .read_findings(&[scheme_finding(&resolution, "wpn_base", Some("cost"))])?;

  assert_eq!(findings[0].finding.kind, LtxFindingKind::Scheme);
  assert_eq!(findings[0].finding.file.as_deref(), Some("system.ltx"));
  assert_eq!(findings[0].line, Some(4));
  assert_eq!(findings[0].finding.section.as_deref(), Some("wpn_base"));
  assert_eq!(findings[0].finding.field.as_deref(), Some("cost"));

  Ok(())
}

#[test]
fn a_finding_with_no_field_anchors_to_the_header() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", WEAPONS)]);
  let resolution: LtxResolution = source.resolve("system.ltx")?;

  let findings: Vec<LtxAnchoredFinding> = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .read_findings(&[scheme_finding(&resolution, "wpn_base", None)])?;

  assert_eq!(findings[0].line, Some(2));

  Ok(())
}

#[test]
fn a_finding_about_an_inherited_field_anchors_to_the_header_that_pulled_it_in() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", WEAPONS)]);
  let resolution: LtxResolution = source.resolve("system.ltx")?;

  let findings: Vec<LtxAnchoredFinding> = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .read_findings(&[scheme_finding(&resolution, "wpn_child", Some("cost"))])?;

  assert_eq!(
    findings[0].line,
    Some(6),
    "wpn_child writes no `cost` line; the header is what brought the value in"
  );
  assert_eq!(findings[0].finding.field.as_deref(), Some("cost"));

  Ok(())
}

#[test]
fn a_finding_about_a_field_the_child_writes_anchors_inside_the_child() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", WEAPONS)]);
  let resolution: LtxResolution = source.resolve("system.ltx")?;

  let findings: Vec<LtxAnchoredFinding> = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .read_findings(&[scheme_finding(&resolution, "wpn_child", Some("description"))])?;

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
  let findings: Vec<LtxAnchoredFinding> = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .read_findings(&[scheme_finding(&resolution, "wpn_base", Some("cost"))])?;

  assert_eq!(findings[0].finding.file.as_deref(), Some("items\\w_base.ltx"));
  assert_eq!(findings[0].line, Some(2));
  assert_eq!(
    findings[0].finding.entry, "system.ltx",
    "still keyed by the root it was found under"
  );

  Ok(())
}

#[test]
fn a_dialect_finding_anchors_to_the_header_of_the_section_it_is_about() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", WEAPONS)]);
  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let finding: LtxFinding = LtxFinding::from_diagnostic(
    "system.ltx",
    &LtxResolutionDiagnostic {
      engine_behaviour: None,
      file: None,
      message: String::from("Inherits 'wpn_none', which is not declared, so it contributes no fields"),
      section: String::from("wpn_child"),
    },
    Some("system.ltx"),
  );

  let findings: Vec<LtxAnchoredFinding> =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_findings(&[finding])?;

  assert_eq!(findings[0].finding.kind, LtxFindingKind::Dialect);
  assert_eq!(findings[0].line, Some(6));

  Ok(())
}

#[test]
fn a_finding_naming_no_config_is_kept_without_a_line() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", WEAPONS)]);
  let resolution: LtxResolution = source.resolve("system.ltx")?;

  let findings: Vec<LtxAnchoredFinding> = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .read_findings(&[LtxFinding::new_unreadable("other.ltx", "Ltx parse error: 1:1 broken")])?;

  assert_eq!(findings[0].finding.kind, LtxFindingKind::Unreadable);
  assert_eq!(findings[0].line, None);

  Ok(())
}

#[test]
fn a_parse_error_carries_its_own_line() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[
    ("system.ltx", "[wpn_base]\ncost = 100\n"),
    ("broken.ltx", "[wpn_base]\ncost = 100\n[unclosed\n"),
  ]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let structure: LtxFileStructure =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_structure("broken.ltx", &[])?;

  let findings: Vec<LtxAnchoredFinding> =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_file_findings(&structure);

  assert_eq!(findings[0].finding.kind, LtxFindingKind::Parse);
  assert_eq!(findings[0].finding.file.as_deref(), Some("broken.ltx"));
  // The parser's own position, carried through untouched - nothing here recomputes where it stopped.
  assert_eq!(findings[0].line, Some(4));

  Ok(())
}

#[test]
fn an_include_that_reached_nothing_is_a_finding_on_its_statement() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", "[wpn_base]\ncost = 100\n#include \"absent.ltx\"\n")]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let structure: LtxFileStructure =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_structure("system.ltx", &[])?;

  let findings: Vec<LtxAnchoredFinding> =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_file_findings(&structure);

  assert_eq!(findings.len(), 1);
  assert_eq!(findings[0].finding.kind, LtxFindingKind::Include);
  assert_eq!(findings[0].line, Some(3));

  Ok(())
}

#[test]
fn keys_stranded_above_a_config_s_first_header_are_a_finding_on_the_first_of_them() -> XrfResult {
  // What the engine does with `loose` here is nothing: Anomaly drops a key read before any header and vanilla passes
  // the null section to `insert_item`, so the resolution this tool builds and the one the game loads differ. Every
  // instance of this shape in either reference tree is a comment somebody wrote without a `;`.
  let source: LtxMapSource = LtxMapSource::new(&[
    ("system.ltx", "#include \"items.ltx\"\n[wpn_base]\ncost = 100\n"),
    ("items.ltx", "; loose\nloose = 1\n\n[wpn_extra]\ncost = 200\n"),
  ]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let reader: LtxRootReader = LtxRootReader::new("system.ltx", "ltx", &resolution, &source);
  let structure: LtxFileStructure = reader.read_structure("items.ltx", &[String::from("system.ltx")])?;

  let findings: Vec<LtxAnchoredFinding> = reader.read_file_findings(&structure);

  assert_eq!(findings.len(), 1);
  assert_eq!(findings[0].finding.kind, LtxFindingKind::RootKeys);
  assert_eq!(findings[0].finding.file.as_deref(), Some("items.ltx"));
  assert_eq!(findings[0].line, Some(2), "the first key written before a header");
  assert!(
    findings[0].finding.engine_behaviour.is_some(),
    "the engine's own answer to be carried"
  );

  Ok(())
}

#[test]
fn an_entry_point_is_judged_for_stranding_keys_like_any_other_config() -> XrfResult {
  // The shape decides it, not whether anything includes the file: a root loaded as an ini loses the keys above its
  // first header exactly as an included config does.
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", "Tronex\n\n[wpn_base]\ncost = 100\n")]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let reader: LtxRootReader = LtxRootReader::new("system.ltx", "ltx", &resolution, &source);
  let structure: LtxFileStructure = reader.read_structure("system.ltx", &[String::from("system.ltx")])?;

  let findings: Vec<LtxAnchoredFinding> = reader.read_file_findings(&structure);

  assert_eq!(findings.len(), 1);
  assert_eq!(findings[0].finding.kind, LtxFindingKind::RootKeys);
  // The header the stranded keys sit above, which is what a reader needs to see where they went.
  assert!(
    findings[0].finding.message.contains("[wpn_base]"),
    "{}",
    findings[0].finding.message
  );

  Ok(())
}

#[test]
fn a_list_config_is_not_judged_for_declaring_no_section() -> XrfResult {
  // The shape root keys are written in on purpose: a roll of names read by the console or by a script rather than
  // loaded as an ini. Judging it would warn about `default_controls.ltx` and every `rspec_*.ltx` in both trees.
  let source: LtxMapSource = LtxMapSource::new(&[("valid_item_sections.ltx", "af_ear\nbandage\n")]);

  let resolution: LtxResolution = source.resolve("valid_item_sections.ltx")?;
  let reader: LtxRootReader = LtxRootReader::new("valid_item_sections.ltx", "ltx", &resolution, &source);
  let structure: LtxFileStructure =
    reader.read_structure("valid_item_sections.ltx", &[String::from("valid_item_sections.ltx")])?;

  assert_eq!(structure.root_entries.len(), 2, "the keys to be read all the same");
  assert!(
    reader.read_file_findings(&structure).is_empty(),
    "a config that declares no section to raise nothing"
  );

  Ok(())
}
