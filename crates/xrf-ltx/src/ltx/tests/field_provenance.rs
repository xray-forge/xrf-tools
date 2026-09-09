//! That a resolved field can say where it is written, and that it costs nothing to a caller who does not ask.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::dialect::{LtxDialect, LtxFieldOrigin, LtxResolution, LtxResolveRequest, LtxStandardDialect};
use crate::source::LtxFilesystemSource;

/// A three-link inheritance chain split across two files, which is the shape a weapon tree has.
///
/// `wpn_ak74` inherits `wpn_rifle`, which inherits `wpn_base`, and only `wpn_base` writes `cost`.
fn write_tree(root: &Path) -> XrfResult<String> {
  fs::create_dir_all(root.join("items"))?;
  fs::write(
    root.join("items").join("w_base.ltx"),
    "[wpn_base]\ncost = 100\nslot = 2\n\n[wpn_rifle]:wpn_base\nslot = 1\nammo = 30\n",
  )?;

  let entry: PathBuf = root.join("system.ltx");

  fs::write(
    &entry,
    "#include \"items\\w_base.ltx\"\n\n[wpn_ak74]:wpn_rifle\nrpm = 600\n",
  )?;

  Ok(entry.to_string_lossy().replace('\\', "/"))
}

/// Resolves the tree written at `root`, asking for provenance.
fn resolve_explained(name: &str) -> XrfResult<LtxResolution> {
  let root: PathBuf = build_absolute_generated_test_resource_path(name);
  let entry: String = write_tree(&root)?;

  LtxStandardDialect.resolve(&entry, &LtxFilesystemSource, LtxResolveRequest::with_provenance())
}

#[test]
fn a_field_written_in_the_section_is_declared_there() -> XrfResult {
  let resolved: LtxResolution = resolve_explained("field_provenance/declared")?;

  assert!(
    matches!(
      resolved.get_origin("wpn_ak74", "rpm"),
      Some(LtxFieldOrigin::Declared { file: Some(file) }) if file.ends_with("system.ltx")
    ),
    "a field written in the section to be declared there, in the file that declared the section"
  );

  Ok(())
}

#[test]
fn an_inherited_field_names_the_section_that_writes_it_rather_than_the_parent_it_arrived_through() -> XrfResult {
  let resolved: LtxResolution = resolve_explained("field_provenance/inherited")?;

  // `wpn_rifle` is the parent named in the header, and it is the wrong answer: it holds `cost` only because it
  // inherited it too. A modder editing `wpn_rifle` would not change this value.
  assert_eq!(
    resolved
      .get_origin("wpn_ak74", "cost")
      .and_then(LtxFieldOrigin::get_inherited_from),
    Some("wpn_base"),
    "an inherited field to name the section it is written in, two links up"
  );

  assert!(
    resolved
      .get_origin("wpn_ak74", "cost")
      .and_then(LtxFieldOrigin::get_file)
      .is_some_and(|file| file.ends_with("w_base.ltx")),
    "and the file that section was declared in, not the file the child was declared in"
  );

  Ok(())
}

#[test]
fn a_parent_field_the_child_overrides_is_declared_by_the_child() -> XrfResult {
  let resolved: LtxResolution = resolve_explained("field_provenance/overridden")?;

  // `wpn_rifle` writes `slot` itself, over the `wpn_base` value it also inherits. The winner is what a value has to
  // account for, so the record follows the field that survives rather than the one it replaced.
  assert!(
    matches!(
      resolved.get_origin("wpn_rifle", "slot"),
      Some(LtxFieldOrigin::Declared { .. })
    ),
    "a field the child rewrites to be declared by the child"
  );

  assert_eq!(
    resolved.ltx.get_from("wpn_rifle", "slot"),
    Some("1"),
    "and the resolved value to be the child's"
  );

  Ok(())
}

#[test]
fn every_resolved_field_of_every_section_is_accounted_for() -> XrfResult {
  let resolved: LtxResolution = resolve_explained("field_provenance/complete")?;

  for (section_name, section) in resolved.ltx.iter() {
    for (key, _) in section {
      assert!(
        resolved.get_origin(section_name, key).is_some(),
        "every resolved field to carry an origin, [{section_name}] {key} did not"
      );
    }
  }

  Ok(())
}

#[test]
fn a_root_field_merged_from_a_second_file_is_attributed_to_the_first() -> XrfResult {
  let root: PathBuf = build_absolute_generated_test_resource_path("field_provenance/root_merge");

  fs::create_dir_all(root.join("items"))?;
  fs::write(
    root.join("items").join("w_base.ltx"),
    "included_root = 1\n\n[wpn_base]\ncost = 100\n",
  )?;

  let entry: PathBuf = root.join("system.ltx");

  fs::write(&entry, "#include \"items\\w_base.ltx\"\n\nentry_root = 2\n")?;

  let resolved: LtxResolution = LtxStandardDialect.resolve(
    &entry.to_string_lossy().replace('\\', "/"),
    &LtxFilesystemSource,
    LtxResolveRequest::with_provenance(),
  )?;

  // The one section that merges instead of colliding, so it is the one section whose fields can come from two files -
  // `merge_sections_from` refuses a duplicate of any named section, which is what makes every other attribution exact.
  // `set_section_origins` stamps only an unstamped section, so the merged root keeps the first file to reach it and
  // both fields report that one. A known limitation rather than an accident: closing it means carrying an origin per
  // root field through the merge, which every resolution would pay for to fix a case the corpus puts in 7 vanilla
  // files, all of them standalone entry points.
  assert_eq!(
    resolved.get_origin("", "entry_root").and_then(LtxFieldOrigin::get_file),
    resolved
      .get_origin("", "included_root")
      .and_then(LtxFieldOrigin::get_file),
    "both root fields to report the same file, which is the first one merged"
  );

  assert!(
    resolved
      .get_origin("", "entry_root")
      .and_then(LtxFieldOrigin::get_file)
      .is_some_and(|file| file.ends_with("w_base.ltx")),
    "and that file to be the included one, because includes merge before the entry's own sections"
  );

  Ok(())
}

#[test]
fn a_resolution_that_was_not_asked_records_nothing() -> XrfResult {
  let root: PathBuf = build_absolute_generated_test_resource_path("field_provenance/unasked");
  let entry: String = write_tree(&root)?;

  let resolved: LtxResolution = LtxStandardDialect.resolve(&entry, &LtxFilesystemSource, LtxResolveRequest::plain())?;

  // The invariant the two tree-wide commands rest on: `ltx verify` and `gamedata verify` resolve every root of a tree
  // and read none of this, so building it would be paid for on every install and read on none.
  assert!(
    resolved.provenance.is_empty(),
    "a plain resolution to record no provenance at all"
  );

  assert!(
    resolved.ltx.get_from("wpn_ak74", "cost").is_some(),
    "while still resolving the same values"
  );

  Ok(())
}
