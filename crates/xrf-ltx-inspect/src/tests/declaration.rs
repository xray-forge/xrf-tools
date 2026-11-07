//! Where a section or a field is written, searched across every config an inventory holds.

use xrf_error::XrfResult;

use crate::declaration::{LtxDeclaration, LtxDeclarationQuery, LtxDeclarationReader, LtxDeclarationStatement};
use crate::inventory::{LtxInventory, LtxInventoryFile, LtxInventoryRole};
use crate::tests::ltx_map_source::LtxMapSource;

/// The base tree, archived, and its patch file beside it, as one project would list them.
struct Project {
  source: LtxMapSource,
  inventory: LtxInventory,
}

/// One config the inventory lists as a loose file.
fn loose(path: &str, role: LtxInventoryRole) -> LtxInventoryFile {
  LtxInventoryFile {
    included_by: Vec::new(),
    is_physical: true,
    path: String::from(path),
    role,
    source: String::from("gamedata"),
  }
}

/// One config the inventory lists as read out of an archive.
fn archived(path: &str, role: LtxInventoryRole) -> LtxInventoryFile {
  LtxInventoryFile {
    is_physical: false,
    source: String::from("database\\gamedata.db"),
    ..loose(path, role)
  }
}

fn project() -> Project {
  let source: LtxMapSource = LtxMapSource::new(&[
    ("items\\w_ak74.ltx", "[wpn_ak74]:wpn_base\ncost = 4000\nrpm = 600\n"),
    (
      "items\\w_base.ltx",
      "[wpn_base]\ncost = 100\nammo_class = ammo_a, ammo_b\n",
    ),
    (
      "mod_system_xxx.ltx",
      "![wpn_ak74]\ncost = 9000\n!rpm\n>ammo_class = ammo_c\n\n!![wpn_unused]\n",
    ),
    ("system.ltx", "#include \"items\\w_*.ltx\"\n\n[Wpn_AK74]\ncost = 1\n"),
  ]);

  let inventory: LtxInventory = LtxInventory {
    files: vec![
      archived("items\\w_ak74.ltx", LtxInventoryRole::Included),
      archived("items\\w_base.ltx", LtxInventoryRole::Included),
      loose("mod_system_xxx.ltx", LtxInventoryRole::Attachment),
      loose("system.ltx", LtxInventoryRole::EntryPoint),
    ],
  };

  Project { source, inventory }
}

fn search(query: LtxDeclarationQuery) -> XrfResult<Vec<LtxDeclaration>> {
  let Project { source, inventory } = project();

  Ok(
    LtxDeclarationReader::new(&inventory, &source)
      .search(&query)?
      .declarations,
  )
}

/// Each match as `path:line`, which is what a person opens.
fn located(declarations: &[LtxDeclaration]) -> Vec<String> {
  declarations
    .iter()
    .map(|declaration| format!("{}:{}", declaration.path, declaration.line))
    .collect()
}

#[test]
fn a_section_is_found_where_it_is_declared_and_where_it_is_patched() -> XrfResult {
  let found: Vec<LtxDeclaration> = search(LtxDeclarationQuery {
    section: Some(String::from("wpn_ak74")),
    ..LtxDeclarationQuery::default()
  })?;

  // `[Wpn_AK74]` in `system.ltx` is a different section: resolution compares names exactly, and so does the search,
  // so a found name is the one `ltx inspect` explains.
  assert_eq!(located(&found), ["items\\w_ak74.ltx:1", "mod_system_xxx.ltx:1"]);

  assert!(!found[0].is_patch);
  assert_eq!(found[0].section_operation, "");
  assert_eq!(found[0].source, "database\\gamedata.db");
  assert!(!found[0].is_physical, "the declaring config is archived");
  assert!(matches!(&found[0].statement, LtxDeclarationStatement::Section { parents } if parents == &["wpn_base"]));

  assert!(found[1].is_patch);
  assert_eq!(found[1].section_operation, "!");
  assert_eq!(found[1].source, "gamedata");

  Ok(())
}

#[test]
fn a_field_keeps_the_operation_it_was_written_with() -> XrfResult {
  let found: Vec<LtxDeclaration> = search(LtxDeclarationQuery {
    section: Some(String::from("wpn_ak74")),
    field: Some(String::from("rpm")),
    value: None,
  })?;

  assert_eq!(located(&found), ["items\\w_ak74.ltx:3", "mod_system_xxx.ltx:3"]);
  assert!(!found[0].is_patch);
  assert!(found[1].is_patch);
  assert!(matches!(
    &found[1].statement,
    LtxDeclarationStatement::Field { name, operation, value: None } if name == "rpm" && operation == "!"
  ));

  Ok(())
}

#[test]
fn a_plain_assignment_under_a_patch_header_is_a_patch() -> XrfResult {
  let found: Vec<LtxDeclaration> = search(LtxDeclarationQuery {
    field: Some(String::from("cost")),
    ..LtxDeclarationQuery::default()
  })?;

  // Every section that writes `cost`, in config order and then document order.
  assert_eq!(
    located(&found),
    [
      "items\\w_ak74.ltx:2",
      "items\\w_base.ltx:2",
      "mod_system_xxx.ltx:2",
      "system.ltx:4"
    ]
  );

  let patched: &LtxDeclaration = &found[2];

  assert!(patched.is_patch, "its header is `![wpn_ak74]`");
  assert_eq!(patched.section_operation, "!");
  assert!(matches!(
    &patched.statement,
    LtxDeclarationStatement::Field { operation, .. } if operation.is_empty()
  ));

  Ok(())
}

#[test]
fn a_value_matches_as_a_substring_and_a_bare_key_never_does() -> XrfResult {
  let found: Vec<LtxDeclaration> = search(LtxDeclarationQuery {
    value: Some(String::from("ammo_")),
    ..LtxDeclarationQuery::default()
  })?;

  assert_eq!(located(&found), ["items\\w_base.ltx:3", "mod_system_xxx.ltx:4"]);

  let narrowed: Vec<LtxDeclaration> = search(LtxDeclarationQuery {
    field: Some(String::from("ammo_class")),
    value: Some(String::from("AMMO_C")),
    ..LtxDeclarationQuery::default()
  })?;

  assert!(narrowed.is_empty(), "a value is compared case-sensitively");

  Ok(())
}

#[test]
fn a_field_query_answers_field_lines_rather_than_their_headers() -> XrfResult {
  let found: Vec<LtxDeclaration> = search(LtxDeclarationQuery {
    section: Some(String::from("wpn_unused")),
    field: Some(String::from("cost")),
    value: None,
  })?;

  // `!![wpn_unused]` is a header with no fields: asked about a field, the header alone is no answer.
  assert!(found.is_empty());

  Ok(())
}

#[test]
fn an_empty_query_matches_nothing() -> XrfResult {
  assert!(search(LtxDeclarationQuery::default())?.is_empty());

  Ok(())
}

#[test]
fn a_config_that_will_not_parse_is_recorded_and_the_search_goes_on() -> XrfResult {
  let source: LtxMapSource = LtxMapSource::new(&[
    ("broken.ltx", "[wpn_broken\ncost = 1\n"),
    ("system.ltx", "[wpn_ak74]\ncost = 4000\n"),
  ]);
  let inventory: LtxInventory = LtxInventory {
    files: vec![
      loose("broken.ltx", LtxInventoryRole::EntryPoint),
      loose("system.ltx", LtxInventoryRole::EntryPoint),
    ],
  };

  let found = LtxDeclarationReader::new(&inventory, &source).search(&LtxDeclarationQuery {
    field: Some(String::from("cost")),
    ..LtxDeclarationQuery::default()
  })?;

  assert_eq!(located(&found.declarations), ["system.ltx:2"]);
  assert_eq!(found.unreadable.len(), 1);
  assert_eq!(found.unreadable[0].path, "broken.ltx");
  assert!(
    !found.unreadable[0].error.message.is_empty(),
    "the parser's reason travels with it"
  );

  Ok(())
}
