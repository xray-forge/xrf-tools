//! What one config as written comes to, once the resolution it belongs to has had its say.

use xrf_error::XrfResult;
use xrf_ltx::LtxResolution;

use crate::ltx_root_reader::LtxRootReader;
use crate::structure::LtxFileStructure;
use crate::tests::ltx_map_source::LtxMapSource;

#[test]
fn a_parent_the_root_holds_resolves_and_one_it_does_not_holds_nothing() -> XrfResult {
  // A config no entry point reaches, which is the only place standard LTX leaves an unresolvable parent standing:
  // resolving a root that declared one would have refused the whole root instead.
  let source: LtxMapSource = LtxMapSource::new(&[
    ("system.ltx", "#include \"items\\w_base.ltx\"\n"),
    ("items\\w_base.ltx", "[wpn_base]\ncost = 100\n"),
    ("items\\w_orphan.ltx", "[wpn_orphan]:wpn_base, wpn_absent\nammo = 30\n"),
  ]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let structure: LtxFileStructure =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_structure("items\\w_orphan.ltx", &[])?;

  let orphan: &_ = &structure.sections[0];

  assert_eq!(orphan.name, "wpn_orphan");
  assert_eq!(orphan.parents[0].name, "wpn_base");
  assert!(orphan.parents[0].resolves);
  assert_eq!(orphan.parents[1].name, "wpn_absent");
  assert!(!orphan.parents[1].resolves, "nothing in the root declares it");

  Ok(())
}

#[test]
fn a_section_with_no_scheme_reports_no_binding() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[(
    "system.ltx",
    "[wpn_bound]\n$scheme = weapon\n\n[wpn_plain]\ncost = 100\n",
  )]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let structure: LtxFileStructure = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .with_declared_schemes(&["weapon"])
    .read_structure("system.ltx", &[])?;

  let bound: &_ = structure.sections[0].scheme.as_ref().expect("a declared binding");

  assert_eq!(bound.name, "weapon");
  assert!(bound.is_declared);
  assert!(structure.sections[1].scheme.is_none(), "nothing binds wpn_plain");

  Ok(())
}

#[test]
fn a_binding_no_scheme_file_declares_is_reported_undeclared() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", "[wpn_bound]\n$scheme = weapon\n")]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let structure: LtxFileStructure =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_structure("system.ltx", &[])?;

  let bound: &_ = structure.sections[0].scheme.as_ref().expect("a binding");

  assert_eq!(bound.name, "weapon");
  assert!(!bound.is_declared);

  Ok(())
}

#[test]
fn an_inherited_binding_is_reported_on_the_child() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[(
    "system.ltx",
    "[wpn_base]\n$scheme = weapon\n\n[wpn_child]:wpn_base\ncost = 100\n",
  )]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let structure: LtxFileStructure = LtxRootReader::new("system.ltx", "ltx", &resolution, &source)
    .with_declared_schemes(&["weapon"])
    .read_structure("system.ltx", &[])?;

  assert_eq!(
    structure.sections[1].scheme.as_ref().map(|scheme| scheme.name.as_str()),
    Some("weapon"),
    "the binding was copied in by inheritance, which is how the verifier reads it too"
  );

  Ok(())
}

#[test]
fn an_include_that_reaches_no_file_resolves_to_nothing() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[
    ("system.ltx", "#include \"present.ltx\"\n#include \"absent.ltx\"\n"),
    ("present.ltx", "[wpn_base]\ncost = 100\n"),
  ]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let structure: LtxFileStructure =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_structure("system.ltx", &[])?;

  assert_eq!(structure.includes[0].statement, "present.ltx");
  assert_eq!(structure.includes[0].resolved, vec![String::from("present.ltx")]);
  assert_eq!(structure.includes[0].line, 1);
  assert_eq!(structure.includes[1].statement, "absent.ltx");
  assert!(structure.includes[1].resolved.is_empty(), "nothing holds it");
  assert_eq!(structure.includes[1].line, 2);

  Ok(())
}

#[test]
fn a_config_that_will_not_parse_still_answers_a_structure() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[
    ("system.ltx", "[wpn_base]\ncost = 100\n"),
    ("broken.ltx", "[wpn_base]\ncost = 100\n[unclosed\n"),
  ]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let structure: LtxFileStructure =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_structure("broken.ltx", &[])?;

  let parse_error: &_ = structure.parse_error.as_ref().expect("the parse to have failed");

  // Where the parser stopped, which for an unclosed header is the end of the input it kept looking through.
  assert_eq!(parse_error.line, 4);
  assert!(structure.sections.is_empty(), "nothing could be read");
  assert!(structure.includes.is_empty());

  Ok(())
}

#[test]
fn a_header_carries_its_line_and_the_entry_points_it_was_asked_about() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", "; header\n\n[wpn_base]\ncost = 100\n")]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let entry_points: Vec<String> = vec![String::from("system.ltx")];
  let structure: LtxFileStructure =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_structure("system.ltx", &entry_points)?;

  assert_eq!(structure.path, "system.ltx");
  assert_eq!(structure.entry_points, entry_points);
  assert_eq!(structure.sections[0].line, 3);
  assert_eq!(structure.sections[0].operation, "", "a plain declaration has no prefix");

  Ok(())
}

#[test]
fn a_wildcard_include_reports_what_it_matched_without_reading_any_of_it() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[
    ("system.ltx", "#include \"items\\w_*.ltx\"\n"),
    ("items\\w_one.ltx", "[one]\ncost = 1\n"),
    ("items\\w_two.ltx", "[two]\ncost = 2\n"),
  ]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let reader: LtxRootReader = LtxRootReader::new("system.ltx", "ltx", &resolution, &source);

  let resolved_at: usize = source.reads();
  let structure: LtxFileStructure = reader.read_structure("system.ltx", &[])?;

  assert_eq!(structure.includes[0].resolved.len(), 2, "both matches to be reported");

  // Expanding a mask is a directory listing, so it already answered which configs exist. Probing each one would read
  // every config a `w_*.ltx` matches to learn what the listing said - hundreds of them on a real weapons tree.
  assert_eq!(
    source.reads() - resolved_at,
    1,
    "only the file being described to be read"
  );

  Ok(())
}

#[test]
fn a_named_include_naming_nothing_reports_nothing() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", "#include \"absent.ltx\"\n")]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let structure: LtxFileStructure =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_structure("system.ltx", &[])?;

  // The other half of the rule above: a statement without a mask expands to itself whether or not anything holds it,
  // so this one has to be probed to know that it reached nothing.
  assert!(
    structure.includes[0].resolved.is_empty(),
    "an include naming an absent config to have reached nothing"
  );

  Ok(())
}
