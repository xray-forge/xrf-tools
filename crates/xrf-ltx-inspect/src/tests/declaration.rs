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
    ..LtxDeclarationQuery::default()
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
    ..LtxDeclarationQuery::default()
  })?;

  // `!![wpn_unused]` is a header with no fields: asked about a field, the header alone is no answer.
  assert!(found.is_empty());

  Ok(())
}

#[test]
fn a_dialect_folding_section_names_finds_every_spelling_of_one() -> XrfResult {
  // Under DLTX `[Wpn_AK74]` is the same section as `[wpn_ak74]`, so a search that missed it would miss a statement the
  // resolver merges. Field names are never folded.
  let found: Vec<LtxDeclaration> = search(LtxDeclarationQuery {
    is_section_case_folded: true,
    ..section("WPN_AK74")
  })?;

  assert_eq!(
    located(&found),
    ["items\\w_ak74.ltx:1", "mod_system_xxx.ltx:1", "system.ltx:3"]
  );

  let field: Vec<LtxDeclaration> = search(LtxDeclarationQuery {
    field: Some(String::from("COST")),
    is_section_case_folded: true,
    ..section("wpn_ak74")
  })?;

  assert!(field.is_empty(), "a field name keeps its case");

  Ok(())
}

#[test]
fn an_empty_query_matches_nothing() -> XrfResult {
  assert!(search(LtxDeclarationQuery::default())?.is_empty());

  Ok(())
}

/// A section query, compared exactly.
fn section(name: &str) -> LtxDeclarationQuery {
  LtxDeclarationQuery {
    section: Some(String::from(name)),
    ..LtxDeclarationQuery::default()
  }
}

/// The entry points a section query narrows to, beside `extra` attachment configs.
///
/// `system.ltx` includes the weapon, `plugins\effects.ltx` declares an unrelated section of the same name, and
/// `magazines\w_am17.ltx` only names it as a parent - where nothing declares that parent, the engine creates it empty.
fn holding_entry_points(query: LtxDeclarationQuery, extra: &[(&str, &str)]) -> XrfResult<Option<Vec<String>>> {
  let mut files: Vec<(&str, &str)> = vec![
    ("items\\w_ak74.ltx", "[wpn_ak74]\ncost = 4000\n"),
    ("magazines\\w_am17.ltx", "[wpn_am17]:wpn_ak74\n"),
    ("other.ltx", "[wpn_other]\ncost = 1\n"),
    ("plugins\\effects.ltx", "[WPN_AK74]:wpn_groza\n"),
    ("system.ltx", "#include \"items\\w_ak74.ltx\"\n"),
  ];
  files.extend_from_slice(extra);

  let source: LtxMapSource = LtxMapSource::new(&files);
  let mut inventory: LtxInventory = LtxInventory {
    files: vec![
      LtxInventoryFile {
        included_by: vec![String::from("system.ltx")],
        ..loose("items\\w_ak74.ltx", LtxInventoryRole::Included)
      },
      loose("magazines\\w_am17.ltx", LtxInventoryRole::EntryPoint),
      loose("other.ltx", LtxInventoryRole::EntryPoint),
      loose("plugins\\effects.ltx", LtxInventoryRole::EntryPoint),
      loose("system.ltx", LtxInventoryRole::EntryPoint),
    ],
  };

  for (path, _) in extra {
    inventory.files.push(loose(path, LtxInventoryRole::Attachment));
  }

  LtxDeclarationReader::new(&inventory, &source).list_holding_entry_points(&query)
}

/// Owned paths, for comparing with what the reader answers.
fn paths(paths: &[&str]) -> Option<Vec<String>> {
  Some(paths.iter().map(|path| String::from(*path)).collect())
}

#[test]
fn a_section_narrows_to_the_entry_points_above_the_configs_writing_it() -> XrfResult {
  // `other.ltx` writes nothing by that name, so resolving it could never answer; an included config answers through
  // the entry point that includes it; `[WPN_AK74]` is another section where names are compared exactly.
  assert_eq!(
    holding_entry_points(section("wpn_ak74"), &[])?,
    paths(&["magazines\\w_am17.ltx", "system.ltx"])
  );
  assert_eq!(holding_entry_points(section("wpn_absent"), &[])?, Some(Vec::new()));

  Ok(())
}

#[test]
fn a_dialect_folding_section_names_narrows_to_every_spelling() -> XrfResult {
  let folded: LtxDeclarationQuery = LtxDeclarationQuery {
    is_section_case_folded: true,
    ..section("wpn_ak74")
  };

  assert_eq!(
    holding_entry_points(folded, &[])?,
    paths(&["magazines\\w_am17.ltx", "plugins\\effects.ltx", "system.ltx"])
  );

  Ok(())
}

#[test]
fn an_attachment_editing_a_section_narrows_nothing_away() -> XrfResult {
  // `![wpn_ak74]` only edits a section some config already declares, so the declaring roots still bound the answer.
  assert_eq!(
    holding_entry_points(
      section("wpn_ak74"),
      &[("mod_system_a.ltx", "![wpn_ak74]\ncost = 9000\n")]
    )?,
    paths(&["magazines\\w_am17.ltx", "system.ltx"])
  );

  Ok(())
}

#[test]
fn an_attachment_that_can_create_a_section_names_no_bound() -> XrfResult {
  // The root an attachment patches is not recorded, so a section it creates could be in any of them - by declaring
  // it, by `@`, or by naming it as a parent nothing in that root declares.
  for patch in ["@[wpn_new]\n", "[wpn_new]\n", "![wpn_ak74]:wpn_new\n"] {
    assert_eq!(
      holding_entry_points(section("wpn_new"), &[("mod_system_a.ltx", patch)])?,
      None,
      "{patch}"
    );
  }

  assert_eq!(
    holding_entry_points(LtxDeclarationQuery::default(), &[])?,
    None,
    "a query naming no section bounds nothing"
  );

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
