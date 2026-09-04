//! One test per row of the DLTX compatibility matrix.
use xrf_error::XrfResult;

use crate::dltx_map_source::DltxMapSource;
use crate::dltx_resolver::{DltxResolved, DltxResolver};
use crate::dltx_severity::DltxSeverity;
use crate::dltx_stores::DltxStores;

/// Loads and resolves a config tree rooted at `root`.
fn resolve(root: &str, files: &[(&str, &str)]) -> XrfResult<DltxResolved> {
  let source: DltxMapSource = DltxMapSource::new(files)?;

  DltxResolver::new(&DltxStores::load(&source, root)?).resolve_all()
}

/// Every warning message, so a test can assert what was said without depending on order.
fn warnings(resolved: &DltxResolved) -> Vec<String> {
  resolved
    .diagnostics
    .iter()
    .filter(|diagnostic| diagnostic.severity == DltxSeverity::Warning)
    .map(|diagnostic| diagnostic.message.clone())
    .collect()
}

#[test]
fn a_plain_section_resolves_to_its_own_fields() -> XrfResult {
  let resolved: DltxResolved = resolve("system.ltx", &[("system.ltx", "[wpn_ak74]\ncost = 4000\nammo = 30\n")])?;

  assert_eq!(resolved.get("wpn_ak74", "cost"), Some("4000"));
  assert_eq!(resolved.get("wpn_ak74", "ammo"), Some("30"));

  Ok(())
}

#[test]
fn an_override_replaces_a_field_and_adds_another() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[wpn_ak74]\ncost = 4000\nammo = 30\n"),
      ("mod_system_a.ltx", "![wpn_ak74]\ncost = 9000\nrare = true\n"),
    ],
  )?;

  assert_eq!(resolved.get("wpn_ak74", "cost"), Some("9000"), "the mod file wins");
  assert_eq!(resolved.get("wpn_ak74", "ammo"), Some("30"), "untouched fields survive");
  assert_eq!(resolved.get("wpn_ak74", "rare"), Some("true"));

  Ok(())
}

#[test]
fn an_override_of_a_section_nothing_declares_changes_nothing() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[present]\na = 1\n"),
      ("mod_system_a.ltx", "![absent]\nb = 2\n"),
    ],
  )?;

  assert_eq!(resolved.list_sections(), vec!["present"]);
  assert!(
    warnings(&resolved).iter().any(|message| message.contains("![absent]")),
    "{:?}",
    warnings(&resolved)
  );

  Ok(())
}

#[test]
fn a_safe_override_creates_the_section_it_cannot_find() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[present]\na = 1\n"),
      ("mod_system_a.ltx", "@[created]\nb = 2\n"),
    ],
  )?;

  // The one thing `@` does that `!` does not: an empty base section is created first, so the override lands.
  assert_eq!(resolved.get("created", "b"), Some("2"));
  assert_eq!(resolved.list_sections(), vec!["created", "present"]);

  Ok(())
}

#[test]
fn a_field_deletion_removes_it_from_the_result() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[wpn_ak74]\ncost = 4000\nammo = 30\n"),
      ("mod_system_a.ltx", "![wpn_ak74]\n!cost\n"),
    ],
  )?;

  assert_eq!(resolved.get("wpn_ak74", "cost"), None);
  assert_eq!(resolved.get("wpn_ak74", "ammo"), Some("30"));

  Ok(())
}

#[test]
fn a_field_deletion_discards_whatever_was_written_after_it() -> XrfResult {
  // `!key = something` deletes; the value is thrown away rather than assigned (`Xr_ini.cpp:867`).
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\nkey = original\n"),
      ("mod_system_a.ltx", "![s]\n!key = ignored\n"),
    ],
  )?;

  assert_eq!(resolved.get("s", "key"), None);

  Ok(())
}

#[test]
fn a_section_deletion_removes_it_after_everything_else_resolves() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[parent]\na = 1\n\n[child]:parent\nb = 2\n"),
      ("mod_system_a.ltx", "!![parent]\n"),
    ],
  )?;

  // The parent is gone, and the child still carries what it inherited: deletion happens after inheritance, not
  // before (`Xr_ini.cpp:1355-1370`).
  assert_eq!(resolved.list_sections(), vec!["child"]);
  assert_eq!(resolved.get("child", "a"), Some("1"));
  assert_eq!(resolved.get("child", "b"), Some("2"));

  Ok(())
}

#[test]
fn a_list_append_adds_to_a_comma_list() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\nammo_class = ammo_a, ammo_b\n"),
      ("mod_system_a.ltx", "![s]\n>ammo_class = ammo_c\n"),
    ],
  )?;

  // Rejoined with commas and no spaces, which is how the engine writes them back.
  assert_eq!(resolved.get("s", "ammo_class"), Some("ammo_a,ammo_b,ammo_c"));

  Ok(())
}

#[test]
fn a_list_remove_drops_every_matching_element() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\nammo_class = a, b, a, c\n"),
      ("mod_system_a.ltx", "![s]\n<ammo_class = a\n"),
    ],
  )?;

  assert_eq!(
    resolved.get("s", "ammo_class"),
    Some("b,c"),
    "all matches, not the first"
  );

  Ok(())
}

#[test]
fn a_list_append_does_not_de_duplicate() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\nlist = a\n"),
      ("mod_system_a.ltx", "![s]\n>list = a\n"),
    ],
  )?;

  assert_eq!(
    resolved.get("s", "list"),
    Some("a,a"),
    "the engine appends without checking"
  );

  Ok(())
}

#[test]
fn a_list_edited_down_to_nothing_drops_its_key() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\nlist = a\nkept = 1\n"),
      ("mod_system_a.ltx", "![s]\n<list = a\n"),
    ],
  )?;

  assert_eq!(resolved.get("s", "list"), None, "an empty list is no key at all");
  assert_eq!(resolved.get("s", "kept"), Some("1"));

  Ok(())
}

#[test]
fn a_list_operation_creates_a_key_that_did_not_exist() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\na = 1\n"),
      ("mod_system_a.ltx", "![s]\n>fresh = x, y\n"),
    ],
  )?;

  assert_eq!(resolved.get("s", "fresh"), Some("x,y"));

  Ok(())
}

#[test]
fn a_deleted_key_cannot_be_revived_by_a_list_operation() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\nlist = a\n"),
      ("mod_system_a.ltx", "![s]\n!list\n>list = b\n"),
    ],
  )?;

  // The deletion is recorded, so the operation is consumed and emits nothing (`Xr_ini.cpp:1229-1238`).
  assert_eq!(resolved.get("s", "list"), None);

  Ok(())
}

#[test]
fn list_operations_apply_in_load_order_and_accumulate() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\nlist = a\n"),
      ("mod_system_aaa.ltx", "![s]\n>list = b\n"),
      ("mod_system_zzz.ltx", "![s]\n>list = c\n"),
    ],
  )?;

  assert_eq!(resolved.get("s", "list"), Some("a,b,c"));

  Ok(())
}

#[test]
fn an_empty_list_operation_value_is_skipped() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\nlist = a\n"),
      ("mod_system_a.ltx", "![s]\n>list =\n"),
    ],
  )?;

  assert_eq!(resolved.get("s", "list"), Some("a"), "nothing was appended");

  Ok(())
}

#[test]
fn the_alphabetically_last_mod_file_wins_a_conflict() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\nkey = base\n"),
      ("mod_system_aaa.ltx", "![s]\nkey = first\n"),
      ("mod_system_zzz.ltx", "![s]\nkey = last\n"),
    ],
  )?;

  // Depth decreases as the list advances and lower wins, so ordering is by name and the last one takes it.
  assert_eq!(resolved.get("s", "key"), Some("last"));

  Ok(())
}

#[test]
fn the_root_file_beats_a_file_it_includes_whatever_the_line_order() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "#include \"included.ltx\"\n\n![s]\nkey = root\n"),
      ("included.ltx", "[s]\nkey = included\n"),
    ],
  )?;

  // Priority is by depth, not by position, which is where DLTX parts company with vanilla LTX: vanilla would take
  // whichever line was read last (`Xr_ini.cpp:698,712,424`).
  assert_eq!(resolved.get("s", "key"), Some("root"));

  Ok(())
}

#[test]
fn a_mod_file_include_outranks_the_base_tree_and_loses_to_its_own_parent() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\nfrom_base = 1\nkey = base\n"),
      ("mod_system_a.ltx", "#include \"patch_extra.ltx\"\n\n![s]\nkey = mod\n"),
      ("patch_extra.ltx", "![s]\nkey = mod_include\nonly_here = yes\n"),
    ],
  )?;

  assert_eq!(
    resolved.get("s", "key"),
    Some("mod"),
    "the mod file beats its own include"
  );
  assert_eq!(
    resolved.get("s", "only_here"),
    Some("yes"),
    "and the include still contributes"
  );
  assert_eq!(resolved.get("s", "from_base"), Some("1"));

  Ok(())
}

#[test]
fn a_later_line_in_one_file_wins_over_an_earlier_one() -> XrfResult {
  let resolved: DltxResolved = resolve("system.ltx", &[("system.ltx", "[s]\nkey = first\nkey = second\n")])?;

  // Equal depth, so the higher position takes it.
  assert_eq!(resolved.get("s", "key"), Some("second"));

  Ok(())
}

#[test]
fn a_child_inherits_its_parent_and_overrides_what_it_declares() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[(
      "system.ltx",
      "[parent]\na = 1\nb = 2\n\n[child]:parent\nb = 20\nc = 3\n",
    )],
  )?;

  assert_eq!(resolved.get("child", "a"), Some("1"));
  assert_eq!(resolved.get("child", "b"), Some("20"));
  assert_eq!(resolved.get("child", "c"), Some("3"));

  Ok(())
}

#[test]
fn inheritance_may_name_a_parent_declared_later() -> XrfResult {
  // Resolution happens after the whole tree is read, so a forward reference is legal where vanilla LTX refuses it.
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[("system.ltx", "[child]:parent\nb = 2\n\n[parent]\na = 1\n")],
  )?;

  assert_eq!(resolved.get("child", "a"), Some("1"));

  Ok(())
}

#[test]
fn a_later_parent_wins_over_an_earlier_one() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[(
      "system.ltx",
      "[first]\nshared = first\n\n[second]\nshared = second\n\n[child]:first,second\n",
    )],
  )?;

  assert_eq!(resolved.get("child", "shared"), Some("second"));

  Ok(())
}

#[test]
fn an_override_can_remove_a_parent() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      (
        "system.ltx",
        "[dropped]\nfrom_dropped = 1\n\n[kept]\nfrom_kept = 2\n\n[child]:dropped,kept\n",
      ),
      ("mod_system_a.ltx", "![child]:!dropped\n"),
    ],
  )?;

  // `!name` is the only parent edit there is; a bare name is the append.
  assert_eq!(resolved.get("child", "from_dropped"), None);
  assert_eq!(resolved.get("child", "from_kept"), Some("2"));

  Ok(())
}

#[test]
fn an_override_added_parent_outranks_the_base_ones() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      (
        "system.ltx",
        "[base_parent]\nshared = base\n\n[mod_parent]\nshared = mod\n\n[child]:base_parent\n",
      ),
      ("mod_system_a.ltx", "![child]:mod_parent\n"),
    ],
  )?;

  // Appended, so it is the last parent and wins.
  assert_eq!(resolved.get("child", "shared"), Some("mod"));

  Ok(())
}

#[test]
fn a_missing_parent_contributes_nothing_and_is_reported() -> XrfResult {
  let resolved: DltxResolved = resolve("system.ltx", &[("system.ltx", "[child]:absent\nown = 1\n")])?;

  // Not an error: the engine fabricates an empty section under that name and carries on.
  assert_eq!(resolved.get("child", "own"), Some("1"));
  assert!(
    warnings(&resolved).iter().any(|message| message.contains("absent")),
    "{:?}",
    warnings(&resolved)
  );

  Ok(())
}

#[test]
fn a_parent_name_is_not_case_folded_while_a_section_name_is() -> XrfResult {
  // The trap from the matrix: `[a]:Base` cannot find `base`, because headers are lowercased and parent tokens are not.
  let resolved: DltxResolved = resolve("system.ltx", &[("system.ltx", "[Base]\na = 1\n\n[child]:Base\n")])?;

  assert_eq!(resolved.get("child", "a"), None, "the parent reference misses");
  assert!(
    resolved.get("base", "a").is_some(),
    "while the section itself is lowercased"
  );

  Ok(())
}

#[test]
fn an_inheritance_cycle_is_an_error_naming_the_chain() -> XrfResult {
  let error: String = resolve("system.ltx", &[("system.ltx", "[a]:b\n\n[b]:c\n\n[c]:a\n")])
    .expect_err("a cycle to be refused")
    .to_string();

  assert!(error.contains("Inheritance cycle"), "{error}");
  assert!(error.contains("->"), "{error}");

  Ok(())
}

#[test]
fn a_duplicate_plain_section_is_an_error_naming_both_files() -> XrfResult {
  let error: String = resolve(
    "system.ltx",
    &[
      ("system.ltx", "#include \"included.ltx\"\n\n[s]\na = 1\n"),
      ("included.ltx", "[s]\na = 2\n"),
    ],
  )
  .expect_err("a duplicate base section to be refused")
  .to_string();

  assert!(error.contains("Duplicate section"), "{error}");
  assert!(error.contains("included.ltx"), "{error}");
  assert!(error.contains("system.ltx"), "{error}");

  Ok(())
}

#[test]
fn a_section_name_is_lowercased_but_a_key_name_is_not() -> XrfResult {
  let resolved: DltxResolved = resolve("system.ltx", &[("system.ltx", "[MixedCase]\nMixedKey = 1\n")])?;

  assert_eq!(resolved.list_sections(), vec!["mixedcase"]);
  assert_eq!(resolved.get("mixedcase", "MixedKey"), Some("1"));
  assert_eq!(resolved.get("mixedcase", "mixedkey"), None);

  Ok(())
}

#[test]
fn a_list_operation_against_a_section_nothing_declares_is_discarded() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[present]\na = 1\n"),
      ("mod_system_a.ltx", "![absent]\n>list = x\n"),
    ],
  )?;

  // Truly silent in the engine, warnings on or off, because only sections with a base are ever evaluated.
  assert_eq!(resolved.list_sections(), vec!["present"]);

  Ok(())
}

#[test]
fn only_the_root_file_is_scanned_for_mod_files() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "#include \"included.ltx\"\n"),
      ("included.ltx", "[s]\nkey = base\n"),
      // A mod file named for the included file, which the engine never looks for.
      ("mod_included_a.ltx", "![s]\nkey = patched\n"),
    ],
  )?;

  assert_eq!(resolved.get("s", "key"), Some("base"));

  Ok(())
}

#[test]
fn provenance_names_the_file_that_won_each_field() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "[s]\nfrom_base = 1\npatched = base\n"),
      ("mod_system_a.ltx", "![s]\npatched = mod\n"),
    ],
  )?;

  let base_origin = resolved.provenance.get("s", "from_base").expect("base field origin");
  let patched_origin = resolved.provenance.get("s", "patched").expect("patched field origin");

  assert_eq!(base_origin.file, "system.ltx");
  assert_eq!(base_origin.depth, 0);
  assert!(!base_origin.is_from_mod_file());

  assert_eq!(patched_origin.file, "mod_system_a.ltx");
  assert_eq!(patched_origin.depth, -200);
  assert!(patched_origin.is_from_mod_file());

  assert_eq!(resolved.provenance.list_patched_fields(), vec![("s", "patched")]);

  Ok(())
}

#[test]
fn a_composite_install_resolves_every_rule_together() -> XrfResult {
  let resolved: DltxResolved = resolve(
    "configs\\system.ltx",
    &[
      (
        "configs\\system.ltx",
        // One wildcard, which matches both weapon files. Naming `w_base.ltx` explicitly as well would load it twice
        // and be refused as a duplicate section, because DLTX has no include guard.
        "#include \"items\\w_*.ltx\"\n\n[system]\nversion = 1\n",
      ),
      (
        "configs\\items\\w_base.ltx",
        "[wpn_base]\ncost = 100\nammo_class = ammo_a, ammo_b\n",
      ),
      ("configs\\items\\w_ak74.ltx", "[wpn_ak74]:wpn_base\ncost = 4000\n"),
      // Alphabetically first, so it loses the `cost` conflict below.
      (
        "configs\\mod_system_aaa.ltx",
        "![wpn_ak74]\ncost = 5000\n>ammo_class = ammo_c\n",
      ),
      (
        "configs\\mod_system_zzz.ltx",
        "![wpn_ak74]\ncost = 9000\n!ammo_class\n\n@[wpn_new]\ncost = 1\n\n!![system]\n",
      ),
    ],
  )?;

  assert_eq!(resolved.get("wpn_ak74", "cost"), Some("9000"), "the last mod file wins");
  assert_eq!(
    resolved.get("wpn_ak74", "ammo_class"),
    None,
    "the deletion in the last file beats the append in the first"
  );
  assert_eq!(resolved.get("wpn_base", "cost"), Some("100"), "the parent is untouched");
  assert_eq!(resolved.get("wpn_new", "cost"), Some("1"), "safe override created it");
  assert_eq!(
    resolved.list_sections(),
    vec!["wpn_ak74", "wpn_base", "wpn_new"],
    "system was deleted, and the rest come out sorted"
  );

  Ok(())
}

#[test]
fn a_tree_with_no_mod_files_resolves_the_same_as_its_base() -> XrfResult {
  // What makes the dialect safe to point at an unpatched install: with nothing to patch, DLTX still answers.
  let resolved: DltxResolved = resolve(
    "system.ltx",
    &[
      ("system.ltx", "#include \"items.ltx\"\n\n[system]\nversion = 1\n"),
      (
        "items.ltx",
        "[wpn_base]\ncost = 100\n\n[wpn_ak74]:wpn_base\ncost = 4000\n",
      ),
    ],
  )?;

  assert_eq!(resolved.get("wpn_ak74", "cost"), Some("4000"));
  assert_eq!(resolved.get("system", "version"), Some("1"));
  assert!(warnings(&resolved).is_empty(), "{:?}", warnings(&resolved));

  Ok(())
}
