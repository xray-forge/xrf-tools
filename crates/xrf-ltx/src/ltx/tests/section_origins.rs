//! That a resolved section remembers the config whose header declared it.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::dialect::LtxStandardDialect;
use crate::ltx::Ltx;

/// A root that declares one section and includes a file declaring two more, one inheriting the other.
fn write_tree(root: &Path) -> XrfResult<String> {
  fs::create_dir_all(root.join("items"))?;
  fs::write(
    root.join("items").join("w_base.ltx"),
    "[wpn_base]\ncost = 100\n\n[wpn_child]:wpn_base\nrpm = 600\n",
  )?;

  let entry: PathBuf = root.join("system.ltx");

  fs::write(&entry, "#include \"items\\w_base.ltx\"\n\n[own]\nkey = value\n")?;

  Ok(entry.to_string_lossy().replace('\\', "/"))
}

#[test]
fn an_included_section_names_the_file_that_declared_it() -> XrfResult {
  let root: PathBuf = build_absolute_generated_test_resource_path("section_origins/included");
  let entry: String = write_tree(&root)?;

  let resolved: Ltx = Ltx::read_from_file_with_dialect(&entry, &LtxStandardDialect)?;

  assert!(
    resolved
      .section("wpn_base")
      .and_then(|it| it.get_origin())
      .is_some_and(|origin| origin.ends_with("w_base.ltx")),
    "an included section to name its own file, not the includer"
  );

  // The entry point's own section still names the entry point, which is what keeps a finding there reading as before.
  assert!(
    resolved
      .section("own")
      .and_then(|it| it.get_origin())
      .is_some_and(|origin| origin.ends_with("system.ltx"))
  );

  fs::remove_dir_all(&root)?;

  Ok(())
}

#[test]
fn inheritance_keeps_the_child_s_own_file_rather_than_its_parent_s() -> XrfResult {
  let root: PathBuf = build_absolute_generated_test_resource_path("section_origins/inherited");
  let entry: String = write_tree(&root)?;

  let resolved: Ltx = Ltx::read_from_file_with_dialect(&entry, &LtxStandardDialect)?;
  let child: &crate::ltx::Section = resolved.section("wpn_child").expect("the inheriting section");

  // Inheritance copies a parent's fields in; it does not move the header that declared the child.
  assert!(child.get_origin().is_some_and(|origin| origin.ends_with("w_base.ltx")));
  assert_eq!(child.get("cost"), Some("100"), "the inherited field to be present");

  fs::remove_dir_all(&root)?;

  Ok(())
}

#[test]
fn a_section_built_in_memory_claims_no_origin() {
  let mut ltx: Ltx = Ltx::new();

  ltx.with_section("made_up").set("key", "value");

  assert_eq!(
    ltx.section("made_up").and_then(|it| it.get_origin()),
    None,
    "a section nothing read to name no file rather than an invented one"
  );
}
