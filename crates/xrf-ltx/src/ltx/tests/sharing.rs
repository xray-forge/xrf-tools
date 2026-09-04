//! That sharing a resolved config's text is invisible to whoever holds it.

use xrf_error::XrfResult;

use crate::ltx::{Ltx, Section};

/// A base section and two children that inherit it, resolved.
fn inherited_tree() -> XrfResult<Ltx> {
  Ltx::read_from_str("[base]\ncost = 100\nrpm = 600\n\n[first]:base\n\n[second]:base\ncost = 200\n")?.into_inherited()
}

#[test]
fn changing_one_section_leaves_the_section_it_inherits_from_alone() -> XrfResult {
  let mut ltx: Ltx = inherited_tree()?;

  // Both hold the same handle for "100" before this, which is the whole point of the storage.
  assert_eq!(ltx.get_from("first", "cost"), Some("100"));
  assert_eq!(ltx.get_from("base", "cost"), Some("100"));

  ltx
    .section_mut("first")
    .expect("the inheriting section to exist")
    .insert("cost", "999");

  assert_eq!(ltx.get_from("first", "cost"), Some("999"));
  assert_eq!(ltx.get_from("base", "cost"), Some("100"), "the parent to be untouched");
  assert_eq!(ltx.get_from("second", "cost"), Some("200"), "a sibling to be untouched");

  Ok(())
}

#[test]
fn changing_a_parent_after_resolution_leaves_the_sections_that_inherited_it_alone() -> XrfResult {
  let mut ltx: Ltx = inherited_tree()?;

  ltx
    .section_mut("base")
    .expect("the base section to exist")
    .insert("rpm", "1");

  assert_eq!(ltx.get_from("base", "rpm"), Some("1"));

  // Inheritance is resolved eagerly, so a child holds what the parent said at the time. Editing the parent afterwards
  // does not reach back into it - which is the same answer the engine gives, since it resolves at load.
  assert_eq!(ltx.get_from("first", "rpm"), Some("600"));

  Ok(())
}

#[test]
fn deleting_a_field_from_one_section_leaves_the_shared_text_readable_elsewhere() -> XrfResult {
  let mut ltx: Ltx = inherited_tree()?;

  assert_eq!(ltx.delete_from("first", "cost").as_deref(), Some("100"));

  assert_eq!(ltx.get_from("first", "cost"), None);
  assert_eq!(ltx.get_from("base", "cost"), Some("100"));

  Ok(())
}

#[test]
fn a_cloned_config_is_independent_of_the_one_it_came_from() -> XrfResult {
  let ltx: Ltx = inherited_tree()?;
  let mut edited: Ltx = ltx.clone();

  // Cheap now - a clone copies handles, not text - which is what makes a copy-on-write editing model affordable. It
  // still has to be a real copy as far as the caller can tell.
  edited
    .section_mut("base")
    .expect("the base section to exist")
    .insert("cost", "0");

  assert_eq!(edited.get_from("base", "cost"), Some("0"));
  assert_eq!(
    ltx.get_from("base", "cost"),
    Some("100"),
    "the original to be untouched"
  );

  Ok(())
}

#[test]
fn a_section_taken_out_of_a_config_can_be_edited_without_touching_it() -> XrfResult {
  let mut ltx: Ltx = inherited_tree()?;
  let mut taken: Section = ltx.delete("second").expect("the section to exist");

  taken.insert("cost", "7");

  assert_eq!(taken.get("cost"), Some("7"));
  assert_eq!(ltx.get_from("base", "cost"), Some("100"));
  assert_eq!(ltx.get_from("first", "cost"), Some("100"));

  Ok(())
}
