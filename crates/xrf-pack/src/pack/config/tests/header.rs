//! What a written `[header]` has to carry, what a named key does to the default, and which entries the engine would
//! read back as something else.

use crate::pack::config::{ArchivePackConfig, ArchivePackHeader, ArchivePackHeaderEntry};

/// One `key=value` pair, as a command line supplies it.
fn entry(key: &str, value: &str) -> ArchivePackHeaderEntry {
  ArchivePackHeaderEntry::new(key, value)
}

/// A loadable header with `extra` written after its required keys.
fn loadable_with(extra: ArchivePackHeaderEntry) -> ArchivePackHeader {
  ArchivePackHeader::default().with_overrides(&[extra])
}

/// The refusal a configuration produced, or a panic when it was accepted.
fn refusal(header: ArchivePackHeader) -> String {
  ArchivePackConfig {
    header: Some(header),
    ..ArchivePackConfig::new("C:\\source", "C:\\out", "patch")
  }
  .validate_for_packing()
  .expect_err("the header is refused")
  .to_string()
}

fn accepts(header: ArchivePackHeader) {
  ArchivePackConfig {
    header: Some(header.clone()),
    ..ArchivePackConfig::new("C:\\source", "C:\\out", "patch")
  }
  .validate_for_packing()
  .unwrap_or_else(|error| panic!("{header:?} is a header the engine reads as written, but was refused: {error}"));
}

#[test]
fn naming_one_key_adds_it_to_the_default_rather_than_replacing_it() {
  // The defect this exists for: `--header creator=me` used to write a header holding only `creator`, and a volume
  // without `auto_load` stops the engine on load rather than mounting oddly.
  let merged: ArchivePackHeader = ArchivePackHeader::default().with_overrides(&[entry("creator", "\"Modder\"")]);
  let keys: Vec<&str> = merged.entries().iter().map(|entry| entry.key.as_str()).collect();

  assert_eq!(keys, ["auto_load", "entry_point", "creator"]);
  assert_eq!(
    merged.entries()[1].value,
    "$fs_root$\\gamedata\\",
    "the entry point Anomaly's own template marks 'do not change !' survives"
  );
}

#[test]
fn naming_a_default_key_overrides_that_one_and_leaves_the_rest() {
  // GSC's own `build_map.ltx` ships `auto_load = false` for multiplayer maps, so the value is the author's to set.
  let merged: ArchivePackHeader = ArchivePackHeader::default().with_overrides(&[entry("Auto_Load", "false")]);

  assert_eq!(
    merged.entries(),
    [
      entry("auto_load", "false"),
      entry("entry_point", "$fs_root$\\gamedata\\")
    ],
    "overriding a key does not add one, and keeps the spelling the engine finds"
  );
}

#[test]
fn the_text_is_one_line_per_entry_under_the_section() {
  // The bytes every volume repeats, as xrCompress wrote them.
  assert_eq!(
    ArchivePackHeader::default().to_text(),
    "[header]\r\nauto_load = true\r\nentry_point = $fs_root$\\gamedata\\\r\n"
  );
}

#[test]
fn the_default_header_is_loadable_as_written() {
  ArchivePackConfig::new("C:\\source", "C:\\out", "patch")
    .validate_for_packing()
    .expect("the default a run writes without being asked must itself be loadable");
}

#[test]
fn a_header_missing_a_key_the_engine_reads_unconditionally_is_refused() {
  // `ProcessArchive` asks for `auto_load` and `LoadArchive` for `entry_point`, both through `r_string`, which ends in
  // `xrDebug::Fatal` when the key is absent.
  let without_auto_load: String = refusal(ArchivePackHeader::new(vec![entry(
    "entry_point",
    "$fs_root$\\gamedata\\",
  )]));
  let without_entry_point: String = refusal(ArchivePackHeader::new(vec![entry("auto_load", "true")]));
  let empty: String = refusal(ArchivePackHeader::new(Vec::new()));

  assert!(
    without_auto_load.contains("names no 'auto_load'"),
    "'{without_auto_load}' names the missing key"
  );
  assert!(
    without_entry_point.contains("names no 'entry_point'"),
    "'{without_entry_point}' names the missing key"
  );
  assert!(
    empty.contains("drop the header entirely"),
    "'{empty}' offers the headerless volume an empty header was probably meant to be"
  );
}

#[test]
fn a_required_key_is_found_only_as_spelled() {
  // `CInifile` finds a key with `strcmp`, so `r_string` misses `Entry_Point`.
  let message: String = refusal(ArchivePackHeader::new(vec![
    entry("auto_load", "true"),
    entry("Entry_Point", "$fs_root$\\gamedata\\"),
  ]));

  assert!(message.contains("names no 'entry_point'"), "'{message}'");
}

#[test]
fn an_entry_point_the_loader_asserts_on_is_refused() {
  // `LoadArchive` special-cases the bare `gamedata` literal and otherwise asserts the value opens with an alias, then
  // reads the path after the alias's first `\`, past the end of a value that has none.
  for entry_point in ["mydata\\", "$game_data$"] {
    let message: String = refusal(loadable_with(entry("entry_point", entry_point)));

    assert!(
      message.contains("is neither 'gamedata' nor an alias"),
      "'{entry_point}': '{message}'"
    );
  }
}

#[test]
fn the_bare_gamedata_literal_and_an_alias_are_both_accepted() {
  for value in ["gamedata", "$fs_root$\\gamedata\\", "$fs_root$\\levels\\"] {
    accepts(loadable_with(entry("entry_point", value)));
  }
}

#[test]
fn an_entry_the_engine_reads_back_differently_is_refused_with_what_it_would_read() {
  for (extra, reading) in [
    // `_parse` drops whitespace outside quotes wherever it stands.
    (entry("creator", "My Mod"), "'creator = MyMod'"),
    (entry("creator", " padded"), "'creator = padded'"),
    // A comment opens at the first `;`, or at a `//`, outside quotes.
    (entry("creator", "me ; and you"), "'creator = me'"),
    (entry("link", "http://moddb.com"), "'link = http:'"),
    // A line break ends the line, so the rest is a line of its own.
    (
      entry("creator", "me\r\nentry_point = gamedata"),
      "'creator = me' and 'entry_point = gamedata'",
    ),
    // The engine splits at the first `=`, and a line opening `[` names a section.
    (entry("a=b", "c"), "'a = b=c'"),
    (entry("[other]", "c"), "no entry at all"),
    (entry("", "c"), "no entry at all"),
  ] {
    let message: String = refusal(loadable_with(extra.clone()));

    assert!(
      message.contains(&format!("would reach the engine as {reading}")),
      "{extra:?}: '{message}'"
    );
  }
}

#[test]
fn quoted_text_reaches_the_engine_as_written() {
  // Quotes are how a value keeps its spaces and comment characters, and the engine keeps the quotes themselves.
  for value in ["\"My Mod\"", "\"GSC; Game World\"", "\"www.moddb.com/mods/my-mod\"", ""] {
    accepts(loadable_with(entry("creator", value)));
  }
}

#[test]
fn a_value_opening_a_quote_it_never_closes_is_refused() {
  // `CInifile` reads on into the following lines to find the closing quote, swallowing the entries written there.
  let message: String = refusal(loadable_with(entry("creator", "\"My Mod")));

  assert!(message.contains("opens a quote it never closes"), "'{message}'");
}

#[test]
fn a_key_named_twice_is_refused_whatever_its_case() {
  let repeated: String = refusal(ArchivePackHeader::new(vec![
    entry("auto_load", "true"),
    entry("entry_point", "gamedata"),
    entry("auto_load", "false"),
  ]));
  let folded: String = refusal(ArchivePackHeader::new(vec![
    entry("auto_load", "true"),
    entry("entry_point", "gamedata"),
    entry("Entry_Point", "$fs_root$\\levels\\"),
  ]));

  assert!(repeated.contains("names 'auto_load' twice"), "'{repeated}'");
  assert!(
    folded.contains("names both 'entry_point' and 'Entry_Point'"),
    "'{folded}'"
  );
}

#[test]
fn a_run_asking_for_no_header_at_all_is_still_allowed() {
  // The documented escape hatch, and the one case where nothing is written for the engine to fail to read.
  ArchivePackConfig {
    header: None,
    ..ArchivePackConfig::new("C:\\source", "C:\\out", "patch")
  }
  .validate_for_packing()
  .expect("a headerless volume is a choice, not a malformed header");
}

#[test]
fn the_header_crosses_the_wire_as_a_plain_list_of_entries() {
  // The JSON configuration files already carry this shape, so they read as they did when the header was text.
  let json: String = serde_json::to_string(&ArchivePackHeader::default()).expect("a header serializes");

  assert_eq!(
    json,
    r#"[{"key":"auto_load","value":"true"},{"key":"entry_point","value":"$fs_root$\\gamedata\\"}]"#
  );
  assert_eq!(
    serde_json::from_str::<ArchivePackHeader>(&json).expect("and reads back"),
    ArchivePackHeader::default()
  );
}
