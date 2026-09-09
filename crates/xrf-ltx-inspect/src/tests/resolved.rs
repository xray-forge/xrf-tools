//! What one resolution comes to, and what it says about where each field was written.

use xrf_error::XrfResult;
use xrf_ltx::LtxResolution;

use crate::ltx_root_reader::LtxRootReader;
use crate::resolved::{LtxResolvedFieldOrigin, LtxResolvedIndex, LtxResolvedSection};
use crate::tests::ltx_map_source::LtxMapSource;

/// The origin of one field of one section, as the record spells it.
fn origin_of<'a>(section: &'a LtxResolvedSection, key: &str) -> &'a LtxResolvedFieldOrigin {
  &section
    .fields
    .iter()
    .find(|field| field.key == key)
    .unwrap_or_else(|| panic!("section [{}] to carry '{key}'", section.name))
    .origin
}

#[test]
fn a_field_two_levels_up_reports_the_section_that_writes_it() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[(
    "system.ltx",
    "[wpn_base]\ncost = 100\n\n[wpn_middle]:wpn_base\nammo = 30\n\n[wpn_child]:wpn_middle\nname = ak\n",
  )]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let sections: Vec<LtxResolvedSection> =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_sections(&["wpn_child"])?;

  let child: &LtxResolvedSection = &sections[0];

  assert_eq!(
    origin_of(child, "cost"),
    &LtxResolvedFieldOrigin::Inherited {
      file: Some(String::from("system.ltx")),
      section: String::from("wpn_base"),
    },
    "wpn_middle is a step on the path, not where the value is written"
  );
  assert_eq!(
    origin_of(child, "ammo"),
    &LtxResolvedFieldOrigin::Inherited {
      file: Some(String::from("system.ltx")),
      section: String::from("wpn_middle"),
    }
  );

  Ok(())
}

#[test]
fn a_field_written_in_the_section_reports_itself_declared() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[(
    "system.ltx",
    "[wpn_base]\ncost = 100\n\n[wpn_child]:wpn_base\nname = ak\n",
  )]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let sections: Vec<LtxResolvedSection> =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_sections(&["wpn_child"])?;

  assert_eq!(
    origin_of(&sections[0], "name"),
    &LtxResolvedFieldOrigin::Declared {
      file: Some(String::from("system.ltx")),
    }
  );

  Ok(())
}

#[test]
fn an_included_config_is_named_as_the_file_a_field_is_written_in() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[
    (
      "system.ltx",
      "#include \"items\\w_base.ltx\"\n\n[wpn_child]:wpn_base\nname = ak\n",
    ),
    ("items\\w_base.ltx", "[wpn_base]\ncost = 100\n"),
  ]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let sections: Vec<LtxResolvedSection> =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_sections(&["wpn_child"])?;

  assert_eq!(
    origin_of(&sections[0], "cost"),
    &LtxResolvedFieldOrigin::Inherited {
      file: Some(String::from("items\\w_base.ltx")),
      section: String::from("wpn_base"),
    },
    "the includer declares the child, the included file declares the parent"
  );

  Ok(())
}

#[test]
fn the_index_carries_declared_parents_that_resolving_flattened_away() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[(
    "system.ltx",
    "[wpn_base]\ncost = 100\n\n[wpn_child]:wpn_base\nname = ak\n",
  )]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let index: LtxResolvedIndex = LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_index()?;

  assert_eq!(index.entry, "system.ltx");
  assert_eq!(index.dialect, "ltx");
  assert_eq!(index.sections[0].name, "wpn_base");
  assert!(index.sections[0].parents.is_empty());
  assert_eq!(index.sections[0].field_count, 1);
  assert_eq!(index.sections[1].name, "wpn_child");
  assert_eq!(index.sections[1].parents, vec![String::from("wpn_base")]);
  assert_eq!(index.sections[1].field_count, 2, "the inherited field is one of them");
  assert_eq!(index.sections[1].origin.as_deref(), Some("system.ltx"));

  Ok(())
}

#[test]
fn a_section_the_root_does_not_hold_is_skipped_rather_than_refused() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[("system.ltx", "[wpn_base]\ncost = 100\n")]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let sections: Vec<LtxResolvedSection> =
    LtxRootReader::new("system.ltx", "ltx", &resolution, &source).read_sections(&["wpn_base", "wpn_absent"])?;

  assert_eq!(sections.len(), 1);
  assert_eq!(sections[0].name, "wpn_base");
  assert_eq!(sections[0].entry, "system.ltx");

  Ok(())
}

#[test]
fn one_reader_reads_each_declaring_config_once_however_many_questions_it_answers() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[
    ("system.ltx", "#include \"items\\w_base.ltx\"\n"),
    (
      "items\\w_base.ltx",
      "[wpn_base]\ncost = 100\n\n[wpn_child]:wpn_base\nrpm = 600\n",
    ),
  ]);

  let resolution: LtxResolution = source.resolve("system.ltx")?;
  let reader: LtxRootReader = LtxRootReader::new("system.ltx", "ltx", &resolution, &source);

  // Resolving did its own reading; only what the reader adds is this test's subject.
  let resolved_at: usize = source.reads();

  reader.read_index()?;

  let indexed_at: usize = source.reads();

  reader.read_sections(&["wpn_base", "wpn_child"])?;

  // A page turn is a lookup, not a tree of parses. Both sections are declared in one config, so indexing reads it once
  // and every later question about that config reads nothing at all.
  assert_eq!(
    indexed_at - resolved_at,
    1,
    "indexing to read the declaring config once"
  );
  assert_eq!(source.reads(), indexed_at, "and a page after it to read nothing");

  Ok(())
}
