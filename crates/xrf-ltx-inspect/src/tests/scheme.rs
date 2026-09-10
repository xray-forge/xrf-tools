//! What judges one section, and how the section measures against it.

use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxResolution, LtxSchemeParser, LtxSectionSchemes};

use crate::ltx_root_reader::LtxRootReader;
use crate::scheme::{LtxSchemeFieldReport, LtxSectionSchemeReport};
use crate::tests::ltx_map_source::LtxMapSource;

/// The declarations one scheme file makes, parsed the way a project parses them.
fn declarations_of(contents: &str) -> XrfResult<LtxSectionSchemes> {
  LtxSchemeParser::parse_from_ltx(&Ltx::read_from_str(contents)?)
}

/// One row of a report, by the field it is about.
fn field_of<'a>(report: &'a LtxSectionSchemeReport, name: &str) -> &'a LtxSchemeFieldReport {
  report
    .fields
    .iter()
    .find(|field| field.name == name)
    .unwrap_or_else(|| panic!("report of [{}] to carry a row for '{name}'", report.section))
}

#[test]
fn a_section_is_judged_by_the_scheme_it_inherits_rather_than_by_its_own_header() -> XrfResult {
  // Nothing in the text of `[wpn_child]:wpn_base` says it is a weapon. The resolution is what knows, which is why the
  // binding is read from there and why the section that writes it is worth naming.
  let source: LtxMapSource = LtxMapSource::new(&[(
    "system.ltx",
    "[wpn_base]\n$scheme = $weapon\ncost = 100\nkind = pistol\n\n[wpn_child]:wpn_base\ncost = 4000\n",
  )]);
  let declarations: LtxSectionSchemes =
    declarations_of("[$weapon]\n$strict = true\ncost = u32\nkind = enum:pistol,rifle\ndescription = ?string\n")?;

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let report: LtxSectionSchemeReport = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .with_declared_schemes(&declarations)
    .read_section_scheme("wpn_child")
    .expect("the root to hold [wpn_child]");

  assert_eq!(report.scheme.as_deref(), Some("$weapon"));
  assert_eq!(report.inherited_from.as_deref(), Some("wpn_base"));
  assert!(report.is_declared);
  assert!(report.is_strict);

  // Supplied here, inherited, and asked for but never written: the three answers a section gives a scheme.
  assert_eq!(
    field_of(&report, "cost").resolved.as_ref().map(|it| it.value.as_str()),
    Some("4000")
  );
  assert_eq!(
    field_of(&report, "kind").resolved.as_ref().map(|it| it.value.as_str()),
    Some("pistol")
  );
  assert!(field_of(&report, "description").resolved.is_none());
  assert!(
    field_of(&report, "description")
      .declared
      .as_ref()
      .is_some_and(|it| it.is_optional)
  );

  Ok(())
}

#[test]
fn a_field_the_scheme_never_named_is_reported_without_a_declaration() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[(
    "system.ltx",
    "[wpn_broken]\n$scheme = $weapon\ncost = 1\nunexpected_field = 1\n",
  )]);
  let declarations: LtxSectionSchemes = declarations_of("[$weapon]\n$strict = true\ncost = u32\n")?;

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let report: LtxSectionSchemeReport = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .with_declared_schemes(&declarations)
    .read_section_scheme("wpn_broken")
    .expect("the root to hold [wpn_broken]");

  assert!(field_of(&report, "unexpected_field").declared.is_none());
  assert_eq!(
    field_of(&report, "unexpected_field")
      .resolved
      .as_ref()
      .map(|it| it.value.as_str()),
    Some("1")
  );
  // Declared first and in the scheme's order, then whatever the section holds beyond them.
  assert_eq!(
    report.fields.iter().map(|it| it.name.as_str()).collect::<Vec<&str>>(),
    vec!["$scheme", "cost", "unexpected_field"]
  );

  Ok(())
}

#[test]
fn the_catch_all_types_the_fields_it_covers_instead_of_standing_as_a_row() -> XrfResult {
  // `* = u32` is not a field anybody wrote, and a reader looking for `level_1` in the scheme file would find nothing.
  // Reporting the catch-all on each row it types is what says where the type came from.
  let source: LtxMapSource = LtxMapSource::new(&[(
    "system.ltx",
    "[grid]\n$scheme = $upgrade_grid\nlevel_1 = 1\nlevel_2 = 2\n",
  )]);
  let declarations: LtxSectionSchemes = declarations_of("[$upgrade_grid]\n$strict = true\n* = u32\n")?;

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let report: LtxSectionSchemeReport = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .with_declared_schemes(&declarations)
    .read_section_scheme("grid")
    .expect("the root to hold [grid]");

  assert!(!report.fields.iter().any(|field| field.name == "*"));
  assert_eq!(
    field_of(&report, "level_1")
      .declared
      .as_ref()
      .map(|it| (it.data_type.as_str(), it.is_any)),
    Some(("u32", true))
  );

  Ok(())
}

#[test]
fn a_section_bound_to_nothing_answers_its_plain_fields() -> XrfResult {
  // The other half of the panel: a section with no `$scheme` is not an error and still has rows worth reading.
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", "[plain]\ncost = 100\nkind = pistol\n")]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let report: LtxSectionSchemeReport = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .read_section_scheme("plain")
    .expect("the root to hold [plain]");

  assert_eq!(report.scheme, None);
  assert!(!report.is_declared);
  assert!(!report.is_strict);
  assert_eq!(
    report.fields.iter().map(|it| it.name.as_str()).collect::<Vec<&str>>(),
    vec!["cost", "kind"]
  );
  assert!(report.fields.iter().all(|field| field.declared.is_none()));

  Ok(())
}

#[test]
fn a_binding_no_scheme_file_declares_is_reported_undeclared() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", "[wpn]\n$scheme = $no_such_scheme\ncost = 1\n")]);
  let declarations: LtxSectionSchemes = declarations_of("[$weapon]\ncost = u32\n")?;

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let report: LtxSectionSchemeReport = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .with_declared_schemes(&declarations)
    .read_section_scheme("wpn")
    .expect("the root to hold [wpn]");

  assert_eq!(report.scheme.as_deref(), Some("$no_such_scheme"));
  assert!(!report.is_declared, "the verifier reports this as a finding of its own");
  // Nothing declares it, so every field the section holds is a plain row rather than a judged one.
  assert!(report.fields.iter().all(|field| field.declared.is_none()));

  Ok(())
}

#[test]
fn a_section_the_root_does_not_hold_is_answered_with_nothing() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", "[wpn]\ncost = 1\n")]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;

  assert!(
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
      .read_section_scheme("missing")
      .is_none()
  );

  Ok(())
}
