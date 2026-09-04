//! The standard-LTX contract, pinned as a whole rather than a rule at a time.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{Ltx, ROOT_SECTION, Section};

/// Writes a config tree under a cleared root and answers that root.
///
/// The root is cleared first because these trees are described in full: a file an earlier run left behind would join a
/// wildcard include and silently widen the golden.
fn tree(name: &str, files: &[(&str, &str)]) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("ltx_golden_contract/{name}"));

  let _ = fs::remove_dir_all(&root);

  for (path, contents) in files {
    let path: PathBuf = root.join(path.replace('\\', "/"));

    fs::create_dir_all(path.parent().expect("entry parent")).expect("config directory");
    fs::write(&path, contents).expect("config file");
  }

  root
}

/// Renders a resolved [`Ltx`] as deterministic text.
///
/// Section order, field order, and the inheritance list are all part of the contract, so they are rendered rather than
/// queried: an assertion over this string fails when any of them moves. Root fields render under a nameless header,
/// which is how the representation stores them.
fn dump(ltx: &Ltx) -> String {
  let mut dumped: String = String::new();

  for name in ltx.sections() {
    let section: &Section = ltx.section(name).expect("listed section to resolve");

    dumped.push('[');
    dumped.push_str(name);
    dumped.push(']');

    if !section.inherited.is_empty() {
      dumped.push(':');
      dumped.push_str(&section.inherited.join(","));
    }

    dumped.push('\n');

    for (key, value) in section {
      dumped.push_str("  ");
      dumped.push_str(key);
      dumped.push_str(" = ");
      dumped.push_str(value);
      dumped.push('\n');
    }
  }

  dumped
}

/// Reads one file of a tree with includes merged and inheritance resolved, and dumps it.
fn dump_full(root: &Path, entry: &str) -> XrfResult<String> {
  Ok(dump(&Ltx::read_from_file_standard(root.join(entry))?))
}

/// Asserts a dump line by line, so a failure names the line that moved instead of printing two long strings.
fn assert_dump(actual: &str, expected: &str) {
  assert_eq!(
    actual.lines().collect::<Vec<&str>>(),
    expected.lines().collect::<Vec<&str>>()
  );
}

/// Asserts a canonical rendering, splitting on the separator the formatter actually emits.
fn assert_formatted(actual: &str, expected: &str) {
  assert_eq!(
    actual.split("\r\n").collect::<Vec<&str>>(),
    expected.split('\n').collect::<Vec<&str>>()
  );
}

/// A vanilla-shaped tree: a root that includes a directory of sections, an inheritance chain, and root fields.
const VANILLA_SYSTEM: &str = "; Vanilla-shaped entry point\r
\r
#include \"sections\\base.ltx\"\r
#include \"sections\\items.ltx\"\r
\r
root_field = root value\r
\r
[main]\r
name = main section ; trailing note\r
count = 4\r
";

const VANILLA_BASE: &str = "[base]\r
hp = 100\r
armor = 10\r
";

const VANILLA_ITEMS: &str = "shared_root = from items\r
\r
[item_base]:base\r
armor = 20\r
cost = 500\r
\r
[item_special]:item_base\r
cost = 900\r
rare = true\r
";

#[test]
fn resolves_a_vanilla_shaped_tree() -> XrfResult {
  let root: PathBuf = tree(
    "vanilla",
    &[
      ("system.ltx", VANILLA_SYSTEM),
      ("sections\\base.ltx", VANILLA_BASE),
      ("sections\\items.ltx", VANILLA_ITEMS),
    ],
  );

  // Included sections come first and in include order, then the entry point's own. Root fields from every file merge
  // into the one nameless section. Inheritance is flattened, so no resolved section keeps a parent list.
  assert_dump(
    &dump_full(&root, "system.ltx")?,
    "[base]
  hp = 100
  armor = 10
[]
  shared_root = from items
  root_field = root value
[item_base]
  hp = 100
  armor = 20
  cost = 500
[item_special]
  hp = 100
  armor = 20
  cost = 900
  rare = true
[main]
  name = main section
  count = 4
",
  );

  Ok(())
}

#[test]
fn formats_a_vanilla_shaped_entry_point() -> XrfResult {
  let root: PathBuf = tree("vanilla_format", &[("system.ltx", VANILLA_SYSTEM)]);

  // Comments, includes, blank runs, and inline notes all survive; only whitespace is normalized, and a blank line is
  // inserted before every section header but the first.
  assert_formatted(
    &Ltx::format_from_file(root.join("system.ltx"))?,
    "; Vanilla-shaped entry point
#include \"sections\\base.ltx\"
#include \"sections\\items.ltx\"
root_field = root value

[main]
name = main section ; trailing note
count = 4
",
  );

  Ok(())
}

/// A CoC-shaped tree: a wildcard include over a directory, which is how CoC and its descendants add sections.
#[test]
fn resolves_a_coc_shaped_wildcard_tree() -> XrfResult {
  let root: PathBuf = tree(
    "coc_wildcard",
    &[
      ("system.ltx", "#include \"items\\w_*.ltx\"\n\n[system]\nversion = 1\n"),
      ("items\\w_ak74.ltx", "[wpn_ak74]\ncost = 4000\n"),
      ("items\\w_pm.ltx", "[wpn_pm]\ncost = 900\n"),
      ("items\\ignored.ltx", "[not_included]\nvalue = 1\n"),
    ],
  );

  // Wildcard matches are sorted, so the merged order does not depend on directory iteration order, and a file the mask
  // does not name is not merged at all.
  assert_dump(
    &dump_full(&root, "system.ltx")?,
    "[wpn_ak74]
  cost = 4000
[wpn_pm]
  cost = 900
[system]
  version = 1
",
  );

  Ok(())
}

/// An Anomaly-shaped tree: includes declared after sections, nested includes, and a deep inheritance chain.
#[test]
fn resolves_an_anomaly_shaped_tree() -> XrfResult {
  let root: PathBuf = tree(
    "anomaly",
    &[
      (
        "system.ltx",
        "[loadout_defaults]\nammo = 30\n\n#include \"npc\\loadouts.ltx\"\n",
      ),
      (
        "npc\\loadouts.ltx",
        "#include \"ranks\\novice.ltx\"\n\n[loadout_base]:loadout_defaults\nweapon = wpn_pm\n",
      ),
      (
        "npc\\ranks\\novice.ltx",
        "[novice]:loadout_base\nweapon = wpn_ak74\nrank = 1\n",
      ),
    ],
  );

  // A nested include resolves against its own directory, and an include declared after a section still merges. The
  // parent of `novice` is declared in the grandparent file, which inheritance resolves after every include is merged.
  //
  // Note the order: resolution emits a parent before the child that names it, which is not the authored order. The
  // merged order here is `novice, loadout_base, loadout_defaults`, because the deepest include merges first; the
  // inheritance pass then walks that order and inserts each parent as it recurses into it. Authored order survives
  // parsing and include merging, not inheritance.
  assert_dump(
    &dump_full(&root, "system.ltx")?,
    "[loadout_defaults]
  ammo = 30
[loadout_base]
  ammo = 30
  weapon = wpn_pm
[novice]
  ammo = 30
  weapon = wpn_ak74
  rank = 1
",
  );

  Ok(())
}

#[test]
fn resolution_reorders_sections_so_a_parent_precedes_its_child() -> XrfResult {
  let parsed: Ltx = Ltx::read_from_str("[child]:parent\nb = 2\n\n[parent]\na = 1\n")?;

  // Parsing keeps the authored order.
  assert_eq!(parsed.sections().collect::<Vec<&str>>(), vec!["child", "parent"]);

  // Resolution does not. This is a consequence of the inheritance walk inserting a parent as it recurses into it, and
  // it is pinned deliberately: writing a resolved document back out would not reproduce the authored file.
  assert_eq!(
    parsed.into_inherited()?.sections().collect::<Vec<&str>>(),
    vec!["parent", "child"]
  );

  Ok(())
}

#[test]
fn merges_root_fields_from_every_included_file_in_include_order() -> XrfResult {
  let root: PathBuf = tree(
    "root_fields",
    &[
      (
        "system.ltx",
        "#include \"first.ltx\"\n#include \"second.ltx\"\nowner = entry\n",
      ),
      ("first.ltx", "shared = first\nfirst_only = 1\n"),
      ("second.ltx", "shared = second\nsecond_only = 2\n"),
    ],
  );

  // Root fields are the one section that merges instead of colliding. A later file overwrites a key an earlier one set,
  // and the key keeps the position of its first appearance.
  assert_dump(
    &dump_full(&root, "system.ltx")?,
    "[]
  shared = second
  first_only = 1
  second_only = 2
  owner = entry
",
  );

  Ok(())
}

#[test]
fn rejects_a_section_declared_in_two_included_files() -> XrfResult {
  let root: PathBuf = tree(
    "duplicate_across_includes",
    &[
      ("system.ltx", "#include \"first.ltx\"\n#include \"second.ltx\"\n"),
      ("first.ltx", "[shared]\nvalue = 1\n"),
      ("second.ltx", "[shared]\nvalue = 2\n"),
    ],
  );

  let error: String = Ltx::read_from_file_standard(root.join("system.ltx"))
    .expect_err("a section declared twice across includes to be refused")
    .to_string();

  assert!(error.contains("duplicate section 'shared'"), "{error}");

  Ok(())
}

#[test]
fn rejects_a_section_declared_twice_in_one_file() {
  let error: String = Ltx::read_from_str("[shared]\na = 1\n\n[shared]\nb = 2\n")
    .expect_err("a section declared twice in one file to be refused")
    .to_string();

  assert!(error.contains("'shared' is declared twice"), "{error}");
}

#[test]
fn rejects_the_same_include_named_twice() {
  let error: String = Ltx::read_from_str("#include \"a.ltx\"\n#include \"a.ltx\"\n")
    .expect_err("the same include named twice to be refused")
    .to_string();

  assert!(error.contains("more than once"), "{error}");
}

#[test]
fn rejects_an_include_that_does_not_name_an_ltx_file() {
  let error: String = Ltx::read_from_str("#include \"config.xml\"\n")
    .expect_err("a non-ltx include to be refused")
    .to_string();

  assert!(error.contains(".ltx extension"), "{error}");
}

#[test]
fn accepts_both_spellings_of_an_include_statement() -> XrfResult {
  let root: PathBuf = tree(
    "include_spellings",
    &[
      (
        "system.ltx",
        "#include \"quoted.ltx\"\n#include(\"parenthesized.ltx\")\n",
      ),
      ("quoted.ltx", "[quoted]\n"),
      ("parenthesized.ltx", "[parenthesized]\n"),
    ],
  );

  let ltx: Ltx = Ltx::read_from_file_included(root.join("system.ltx"))?;

  assert!(ltx.has_section("quoted"));
  assert!(ltx.has_section("parenthesized"));

  Ok(())
}

#[test]
fn treats_a_missing_include_with_a_ts_sibling_as_nothing_to_merge() -> XrfResult {
  let root: PathBuf = tree(
    "ts_sibling",
    &[
      ("system.ltx", "#include \"generated.ltx\"\n[present]\nvalue = 1\n"),
      ("generated.ts", "// the generator's input, not a config\n"),
    ],
  );

  // A config a generator has not produced yet is nothing to merge rather than a failure, which is what lets a project be
  // read mid-build. Without the sibling the same missing include is an error.
  let ltx: Ltx = Ltx::read_from_file_included(root.join("system.ltx"))?;

  assert!(ltx.has_section("present"));
  assert!(!ltx.has_section("generated"));

  Ok(())
}

#[test]
fn reports_a_missing_include_with_no_ts_sibling() -> XrfResult {
  let root: PathBuf = tree("missing_include", &[("system.ltx", "#include \"absent.ltx\"\n")]);

  assert!(Ltx::read_from_file_included(root.join("system.ltx")).is_err());

  Ok(())
}

#[test]
fn refuses_to_inherit_a_section_that_is_declared_nowhere() {
  let error: String = Ltx::read_from_str("[child]:missing\n")
    .expect("parsing records the parent")
    .into_inherited()
    .expect_err("inheriting an undeclared section to be refused")
    .to_string();

  assert!(error.contains("Failed to inherit unknown section [missing]"), "{error}");
}

#[test]
fn refuses_a_section_that_inherits_itself() {
  let error: String = Ltx::read_from_str("[loop]:loop\n")
    .expect("parsing records the parent")
    .into_inherited()
    .expect_err("a self-inheriting section to be refused")
    .to_string();

  assert!(error.contains("cannot inherit self"), "{error}");
}

#[test]
fn a_later_parent_wins_over_an_earlier_one_and_own_keys_win_over_both() -> XrfResult {
  let ltx: Ltx = Ltx::read_from_str(
    "[first]\nshared = first\nonly_first = 1\n\n[second]\nshared = second\nonly_second = 2\n\n[child]:first,second\nshared = own\n",
  )?
  .into_inherited()?;

  assert_dump(
    &dump(&ltx),
    "[first]
  shared = first
  only_first = 1
[second]
  shared = second
  only_second = 2
[child]
  shared = own
  only_first = 1
  only_second = 2
",
  );

  Ok(())
}

#[test]
fn keeps_a_declared_parent_list_out_of_the_resolved_section() -> XrfResult {
  let parsed: Ltx = Ltx::read_from_str("[parent]\na = 1\n\n[child]:parent\nb = 2\n")?;

  assert_eq!(parsed.section("child").expect("child").inherited, vec!["parent"]);

  // Resolution is a flattening: the parent list is spent, not carried, so nothing downstream can inherit twice.
  let resolved: Ltx = parsed.into_inherited()?;

  assert!(resolved.section("child").expect("child").inherited.is_empty());

  Ok(())
}

#[test]
fn honours_the_header_directive_that_skips_inheritance() -> XrfResult {
  let ltx: Ltx = Ltx::read_from_str("; @xrf-ltx skip-inheritance\n[child]:missing\nvalue = 1\n")?.into_inherited()?;

  // The parent stays declared and unresolved, which is the point: a file naming a section the project assembles
  // elsewhere is readable on its own.
  assert_eq!(ltx.section("child").expect("child").inherited, vec!["missing"]);

  Ok(())
}

#[test]
fn trims_keys_values_and_inline_comments() -> XrfResult {
  let ltx: Ltx =
    Ltx::read_from_str("[section]\n   padded_key   =    padded value   ; note\nbare_key\nempty_value =\n")?;
  let section: &Section = ltx.section("section").expect("section");

  assert_eq!(section.get("padded_key"), Some("padded value"));
  assert_eq!(section.get("bare_key"), Some(""));
  assert_eq!(section.get("empty_value"), Some(""));

  Ok(())
}

#[test]
fn a_semicolon_anywhere_in_a_value_starts_a_comment() -> XrfResult {
  let ltx: Ltx = Ltx::read_from_str("[section]\nno_space = a;b\nspaced = value ; dropped\nquoted = \"a;b\"\n")?;
  let section: &Section = ltx.section("section").expect("section");

  // A value is cut at its first semicolon whether or not whitespace precedes it, and quoting does not protect one.
  // The engine's own parser is quote-aware here (`Xr_ini.cpp` `_parse`), so a quoted semicolon is a known divergence
  // rather than a rule to preserve on purpose; it is pinned so a representation change cannot move it silently.
  assert_eq!(section.get("no_space"), Some("a"));
  assert_eq!(section.get("spaced"), Some("value"));
  assert_eq!(section.get("quoted"), Some("\"a"));

  Ok(())
}

#[test]
fn reads_and_writes_windows_1251_bytes_unchanged() -> XrfResult {
  let root: PathBuf = build_absolute_generated_test_resource_path("ltx_golden_contract/encoding");

  let _ = fs::remove_dir_all(&root);
  fs::create_dir_all(&root)?;

  // Cyrillic in Windows-1251, which is what shipped configs carry. Written as bytes because the source file is UTF-8.
  let path: PathBuf = root.join("cyrillic.ltx");
  let contents: Vec<u8> = [
    b"[section]\r\nname = ".as_slice(),
    &[0xCF, 0xF0, 0xE8, 0xE2, 0xE5, 0xF2],
    b"\r\n",
  ]
  .concat();

  fs::write(&path, &contents)?;

  let ltx: Ltx = Ltx::read_from_path(&path)?;

  assert_eq!(ltx.section("section").expect("section").get("name"), Some("Привет"));

  let round_tripped: PathBuf = root.join("round_tripped.ltx");

  ltx.write_to_path(&round_tripped)?;

  assert_eq!(fs::read(&round_tripped)?, contents);

  Ok(())
}

#[test]
fn refuses_a_value_that_windows_1251_cannot_hold() {
  let mut ltx: Ltx = Ltx::new();

  ltx.with_section("section").set("key", "日本語");

  assert!(
    ltx
      .write_to_path(build_absolute_generated_test_resource_path(
        "ltx_golden_contract/refused.ltx"
      ))
      .is_err()
  );
}

#[test]
fn names_the_position_a_parse_failure_was_detected_at() {
  // The position is where the parser stopped, not where the offending statement began: an unterminated header is only
  // known to be one at the end of its line, so a trailing newline puts the report on the following line.
  let with_newline: String = Ltx::read_from_str("[first]\nvalue = 1\n[broken\n")
    .expect_err("a section header with no closing bracket to be refused")
    .to_string();

  assert!(with_newline.contains("4:1"), "{with_newline}");

  let at_eof: String = Ltx::read_from_str("[first]\nvalue = 1\n[broken")
    .expect_err("a section header with no closing bracket to be refused")
    .to_string();

  assert!(at_eof.contains("3:8"), "{at_eof}");
}

#[test]
fn records_the_path_and_directory_a_file_was_read_from() -> XrfResult {
  let root: PathBuf = tree("recorded_paths", &[("nested\\config.ltx", "[section]\n")]);
  let path: PathBuf = root.join("nested/config.ltx");
  let ltx: Ltx = Ltx::read_from_path(&path)?;

  assert_eq!(ltx.path.as_deref(), Some(path.as_path()));
  assert_eq!(ltx.directory.as_deref(), Some(root.join("nested").as_path()));

  Ok(())
}

#[test]
fn keeps_the_authored_order_of_sections_and_of_fields_within_them() -> XrfResult {
  let ltx: Ltx = Ltx::read_from_str("[zebra]\nz = 1\na = 2\n\n[alpha]\nm = 3\nb = 4\n")?;

  // Ordered, never sorted: a formatter rewrite and a spawn export both depend on the authored order.
  assert_eq!(ltx.sections().collect::<Vec<&str>>(), vec!["zebra", "alpha"]);
  assert_eq!(
    ltx
      .section("zebra")
      .expect("zebra")
      .iter()
      .map(|(key, _)| key)
      .collect::<Vec<&str>>(),
    vec!["z", "a"]
  );

  Ok(())
}

#[test]
fn an_already_canonical_file_is_reported_as_formatted() -> XrfResult {
  let canonical: &str = "; note\r\n#include \"other.ltx\"\r\nroot = 1\r\n\r\n[section]\r\nkey = value\r\n";

  assert!(Ltx::is_formatted(canonical.as_bytes())?);
  assert!(!Ltx::is_formatted(b"[section]\nkey=value\n")?);

  Ok(())
}

#[test]
fn an_empty_document_holds_no_sections_at_all() -> XrfResult {
  // Not even the root one: the parser creates a section when a line needs it, so an empty file resolves to an empty
  // document. Reading the root section reports its absence; only asking to write one creates it.
  let mut ltx: Ltx = Ltx::read_from_str("")?;

  assert_eq!(ltx.len(), 0);
  assert!(!ltx.has_section(ROOT_SECTION));
  assert!(ltx.root_section().is_none());

  assert!(ltx.root_section_mut().is_empty());
  assert!(ltx.has_section(ROOT_SECTION));

  Ok(())
}

#[test]
fn a_root_field_creates_the_nameless_section() -> XrfResult {
  let ltx: Ltx = Ltx::read_from_str("root_field = 1\n")?;

  assert_eq!(ltx.sections().collect::<Vec<&str>>(), vec![ROOT_SECTION]);
  assert_eq!(ltx.get_from(ROOT_SECTION, "root_field"), Some("1"));

  Ok(())
}
