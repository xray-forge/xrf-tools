use std::fs;
use std::path::{Path, PathBuf};

use xrf_collections::OrderedMap;
use xrf_error::XrfResult;
use xrf_ltx::Ltx;
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayMountMode, XrayRoots, XraySearch};

use crate::dialog_listed_files::DialogListedFiles;
use crate::partner::dialog_character::DialogCharacter;
use crate::partner::dialog_offer::DialogOffer;
use crate::partner::dialog_partners::DialogPartners;
use crate::partner::dialog_profile::DialogProfile;
use crate::project::dialog_project_mode::DialogProjectMode;

/// The shared include shipped characters take their dialogs from, which declares a start dialog of its own.
const CHARACTER_DIALOGS: &str =
  "<start_dialog>hello_dialog</start_dialog>\r\n<actor_dialog>about_quests</actor_dialog>\r\n";

const CHARACTERS: &str = r#"<?xml version="1.0" encoding="windows-1251"?>
<xml>
  <specific_character id="snag">
    <class>Snag_Class</class>
    <start_dialog>snag_cache_dialog</start_dialog>
#include "gameplay\character_dialogs.xml"
    <actor_dialog>snag_share_dialog</actor_dialog>
  </specific_character>
  <specific_character id="novice_1">
    <class>novice</class>
#include "gameplay\character_dialogs.xml"
  </specific_character>
  <specific_character id="novice_2">
    <class>novice</class>
    <actor_dialog>novice_dialog</actor_dialog>
  </specific_character>
  <specific_character id="novice_story" no_random="1">
    <class>novice</class>
  </specific_character>
</xml>
"#;

const PROFILES: &str = r#"<xml>
  <character id="snag"><class>snag_class</class></character>
  <character id="snag_pinned"><specific_character>snag</specific_character><class>novice</class></character>
  <character id="novice"><class>novice</class></character>
  <character id="anyone"></character>
</xml>
"#;

const INFO_PORTIONS: &str = r#"<game_information_portions>
  <info_portion id="global_dialogs"><dialog>actor_break_dialog</dialog><dialog>about_quests</dialog></info_portion>
  <info_portion id="no_dialogs"><disable>global_dialogs</disable></info_portion>
</game_information_portions>
"#;

/// Lay out a gameplay folder with the shared character include beside the given files, and return its root.
fn lay_out(name: &str, files: &[(&str, &str)]) -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&format!("partner/{name}"))?;
  let gameplay: PathBuf = root.join("configs").join("gameplay");

  fs::create_dir_all(&gameplay)?;
  fs::write(gameplay.join("character_dialogs.xml"), CHARACTER_DIALOGS)?;

  for (file, content) in files {
    fs::write(gameplay.join(file), content)?;
  }

  Ok(root)
}

/// Lay out a gameplay folder and read its partners, choosing the files by name.
fn read_partners(name: &str, files: &[(&str, &str)]) -> XrfResult<DialogPartners> {
  let root: PathBuf = lay_out(name, files)?;
  let partners: DialogPartners = DialogPartners::from_search(&open(&root)?, DialogProjectMode::DIALOGS_PREFIX)?;

  fs::remove_dir_all(root)?;

  Ok(partners)
}

fn open(root: &Path) -> XrfResult<XraySearch> {
  XrayRoots::one(root.to_path_buf(), XrayMountMode::Directory)
    .to_search_plan()?
    .open()
}

/// Read the shipped-shaped tree under a root of its own, since tests run in parallel.
fn read_shipped(name: &str) -> XrfResult<DialogPartners> {
  read_partners(
    name,
    &[
      ("character_desc_zaton.xml", CHARACTERS),
      ("npc_profile.xml", PROFILES),
      ("info_portions.xml", INFO_PORTIONS),
    ],
  )
}

fn profile<'a>(partners: &'a DialogPartners, id: &str) -> &'a DialogProfile {
  partners.find_profile(id).expect("Expected the profile to be read")
}

fn resolve(partners: &DialogPartners, id: &str) -> Vec<String> {
  partners
    .resolve_profile(profile(partners, id))
    .into_iter()
    .map(|character| character.get_id().to_owned())
    .collect()
}

#[test]
fn reads_characters_with_their_includes_spliced_in() -> XrfResult {
  let partners: DialogPartners = read_shipped("characters")?;
  let snag: &DialogCharacter = &partners.get_characters()["snag"];

  assert_eq!(snag.get_class(), Some("snag_class"));
  // The include declares `hello_dialog` after the character's own, and the first declaration is the one read.
  assert_eq!(snag.get_start_dialog(), Some("snag_cache_dialog"));
  assert_eq!(snag.get_actor_dialogs(), ["about_quests", "snag_share_dialog"]);
  assert!(snag.is_random());

  assert_eq!(
    partners.get_characters()["novice_1"].get_start_dialog(),
    Some("hello_dialog")
  );
  assert!(!partners.get_characters()["novice_story"].is_random());
  assert!(partners.get_findings().is_empty());

  Ok(())
}

#[test]
fn resolves_a_profile_to_its_pinned_character_or_to_the_random_characters_of_its_class() -> XrfResult {
  let partners: DialogPartners = read_shipped("profiles")?;

  assert_eq!(resolve(&partners, "snag"), ["snag"]);
  assert_eq!(resolve(&partners, "snag_pinned"), ["snag"]);
  assert_eq!(resolve(&partners, "novice"), ["novice_1", "novice_2"]);
  assert_eq!(resolve(&partners, "anyone"), ["snag", "novice_1", "novice_2"]);

  Ok(())
}

#[test]
fn resolves_a_class_profile_only_through_a_team_default_character_of_its_class() -> XrfResult {
  let partners: DialogPartners = read_partners(
    "team_default",
    &[
      (
        "character_desc_a.xml",
        r#"<xml>
  <specific_character id="pinned" no_random="1"><class>story</class></specific_character>
  <specific_character id="fallback" team_default="1"><class>novice</class></specific_character>
  <specific_character id="plain"><class>novice</class></specific_character>
  <specific_character id="lone"><class>loner</class></specific_character>
</xml>"#,
      ),
      (
        "npc_profile.xml",
        r#"<xml>
  <character id="story"><specific_character>pinned</specific_character></character>
  <character id="lost"><specific_character>nobody</specific_character></character>
  <character id="novice"><class>novice</class></character>
  <character id="loner"><class>loner</class></character>
  <character id="story_class"><class>story</class></character>
</xml>"#,
      ),
    ],
  )?;
  let is_resolvable = |id: &str| partners.is_profile_resolvable(profile(&partners, id));

  assert!(partners.get_characters()["fallback"].is_team_default());
  assert!(!partners.get_characters()["plain"].is_team_default());
  assert_eq!(
    partners.get_characters()["plain"].get_logical_path(),
    r"configs\gameplay\character_desc_a.xml"
  );

  // A pinned character needs only to be declared, `no_random` or not.
  assert!(is_resolvable("story"));
  assert!(!is_resolvable("lost"));
  assert!(is_resolvable("novice"));
  // A class with random characters and no team default one is what the engine asserts on.
  assert_eq!(resolve(&partners, "loner"), ["lone"]);
  assert!(!is_resolvable("loner"));
  // A class only a `no_random` character has picks nothing.
  assert!(!is_resolvable("story_class"));

  Ok(())
}

#[test]
fn lists_start_then_actor_then_info_portion_dialogs_with_every_way_each_is_offered() -> XrfResult {
  let partners: DialogPartners = read_shipped("offers")?;
  let offers: OrderedMap<String, Vec<DialogOffer>> = partners.list_offers(profile(&partners, "snag"));
  let snag = || String::from("snag");

  assert_eq!(
    offers.keys().map(String::as_str).collect::<Vec<&str>>(),
    [
      "snag_cache_dialog",
      "about_quests",
      "snag_share_dialog",
      "actor_break_dialog"
    ]
  );
  assert_eq!(offers["snag_cache_dialog"], [DialogOffer::Start { character: snag() }]);
  assert_eq!(
    offers["about_quests"],
    [
      DialogOffer::Actor { character: snag() },
      DialogOffer::Info {
        info: String::from("global_dialogs")
      }
    ]
  );
  assert!(partners.get_info_portions()["no_dialogs"].get_dialogs().is_empty());

  Ok(())
}

#[test]
fn keeps_the_first_declaration_of_an_id_and_reports_the_others() -> XrfResult {
  let partners: DialogPartners = read_partners(
    "duplicate",
    &[
      (
        "character_desc_a.xml",
        r#"<xml><specific_character id="x"><class>a</class></specific_character></xml>"#,
      ),
      (
        "character_desc_b.xml",
        r#"<xml><specific_character id="x"><class>b</class></specific_character></xml>"#,
      ),
    ],
  )?;

  assert_eq!(partners.get_characters()["x"].get_class(), Some("a"));
  assert_eq!(partners.get_findings().len(), 1);
  assert_eq!(partners.get_findings()[0].rule, DialogPartners::DUPLICATE_RULE);
  assert_eq!(
    partners.get_findings()[0].subject.as_deref(),
    Some(r"configs\gameplay\character_desc_b.xml")
  );
  assert!(partners.get_findings()[0].message.starts_with("Character 'x'"));

  Ok(())
}

// The engine's id index skips an element whose id is empty, so it neither stands as a character nor duplicates one.
#[test]
fn skips_a_character_or_profile_whose_id_is_empty() -> XrfResult {
  let partners: DialogPartners = read_partners(
    "empty_id",
    &[
      (
        "character_desc_a.xml",
        r#"<xml><specific_character id=""><class>a</class></specific_character></xml>"#,
      ),
      (
        "character_desc_b.xml",
        r#"<xml><specific_character id=""><class>b</class></specific_character></xml>"#,
      ),
      (
        "npc_profile.xml",
        r#"<xml><character id=""><class>a</class></character></xml>"#,
      ),
    ],
  )?;

  assert!(partners.get_characters().is_empty());
  assert!(partners.find_profile("").is_none());
  assert!(partners.get_findings().is_empty());

  Ok(())
}

#[test]
fn reports_a_file_it_cannot_read_and_reads_the_rest() -> XrfResult {
  let partners: DialogPartners = read_partners(
    "unreadable",
    &[
      (
        "character_desc_a.xml",
        "<xml>\n#include \"gameplay\\missing.xml\"\n</xml>",
      ),
      ("npc_profile.xml", PROFILES),
      // Neither a character, a profile nor an info portion file, so it is not read at all.
      ("dialogs.xml", "<game_dialogs>"),
    ],
  )?;

  assert!(partners.get_characters().is_empty());
  assert_eq!(partners.get_profiles().len(), 4);
  assert_eq!(partners.get_findings().len(), 1);
  assert_eq!(partners.get_findings()[0].rule, "dialog.partner-unreadable");
  assert!(partners.get_findings()[0].message.contains("missing.xml"));

  Ok(())
}

/// The engine reads only the files `system.ltx` lists, so a file the lists leave out declares nothing.
#[test]
fn reads_only_the_listed_files_and_reports_a_missing_one() -> XrfResult {
  let root: PathBuf = lay_out(
    "listed",
    &[
      ("character_desc_zaton.xml", CHARACTERS),
      ("npc_profile.xml", PROFILES),
      (
        "npc_profile_unlisted.xml",
        r#"<xml><character id="stray"><class>novice</class></character></xml>"#,
      ),
    ],
  )?;
  let listed: DialogListedFiles = DialogListedFiles {
    characters: vec![String::from("character_desc_zaton")],
    profiles: vec![String::from("npc_profile"), String::from("npc_profile_gone")],
    ..DialogListedFiles::default()
  };
  let partners: DialogPartners = DialogPartners::from_listed(&open(&root)?, DialogProjectMode::DIALOGS_PREFIX, &listed);

  fs::remove_dir_all(root)?;

  assert!(partners.find_profile("snag").is_some());
  assert!(partners.find_profile("stray").is_none());
  assert_eq!(partners.get_findings().len(), 1);
  assert_eq!(partners.get_findings()[0].rule, DialogPartners::MISSING_RULE);
  assert_eq!(
    partners.get_findings()[0].subject.as_deref(),
    Some(r"configs\gameplay\npc_profile_gone.xml")
  );

  Ok(())
}

/// `system.ltx` lists each kind under its own key, a list it leaves out empty; one listing none names no lists at all.
#[test]
fn reads_the_lists_system_ltx_holds() -> XrfResult {
  let system: Ltx = Ltx::read_from_str(
    "[dialogs]\nfiles = dialogs, dialogs_zaton,\n\n[profiles]\nfiles = npc_profile\nspecific_characters_files = \
     character_desc_general\n",
  )?;

  assert_eq!(
    DialogListedFiles::from_system_ltx(&system),
    Some(DialogListedFiles {
      dialogs: vec![String::from("dialogs"), String::from("dialogs_zaton")],
      characters: vec![String::from("character_desc_general")],
      profiles: vec![String::from("npc_profile")],
      info_portions: Vec::new(),
    })
  );
  assert_eq!(
    DialogListedFiles::from_system_ltx(&Ltx::read_from_str("[actor]\nclass = O_ACTOR\n")?),
    None
  );

  Ok(())
}

/// The files a caller holding the lists compares them against: every partner file by name, but not a part another
/// includes, which the engine reads through its includer.
#[test]
fn lists_partner_files_by_name_without_the_parts_they_include() -> XrfResult {
  let root: PathBuf = lay_out(
    "named",
    &[
      (
        "character_desc_general.xml",
        "<xml>\n#include \"gameplay\\character_desc_general_part.xml\"\n</xml>",
      ),
      ("character_desc_general_part.xml", "<specific_character id=\"a\"/>\n"),
      ("info_portions.xml", INFO_PORTIONS),
      ("dialogs.xml", "<game_dialogs/>"),
    ],
  )?;
  let named: Vec<String> = DialogPartners::list_named_files(&open(&root)?, DialogProjectMode::DIALOGS_PREFIX)?;

  fs::remove_dir_all(root)?;

  assert_eq!(
    named,
    [
      r"configs\gameplay\character_desc_general.xml",
      r"configs\gameplay\info_portions.xml"
    ]
  );

  Ok(())
}

#[test]
fn reads_a_part_another_file_includes_only_through_that_file() -> XrfResult {
  // Anomaly splits its characters into parts with no root, each carrying a declaration, and includes them.
  let partners: DialogPartners = read_partners(
    "parts",
    &[
      (
        "character_desc_general.xml",
        "<xml>\n#include \"gameplay\\character_desc_general_freedom.xml\"\n</xml>",
      ),
      (
        "character_desc_general_freedom.xml",
        "<?xml version=\"1.0\" encoding=\"windows-1251\"?>\n<specific_character id=\"a\"><class>c</class></specific_character>\n<specific_character id=\"b\"><class>c</class></specific_character>\n",
      ),
    ],
  )?;

  assert_eq!(
    partners
      .get_characters()
      .keys()
      .map(String::as_str)
      .collect::<Vec<&str>>(),
    ["a", "b"]
  );
  assert!(partners.get_findings().is_empty(), "{:?}", partners.get_findings());

  Ok(())
}

#[test]
fn recognizes_a_part_whatever_case_and_separators_its_include_spells() -> XrfResult {
  let partners: DialogPartners = read_partners(
    "part_spelling",
    &[
      (
        "character_desc_general.xml",
        "<xml>\n#include \"Gameplay/Character_Desc_Part.xml\"\n</xml>",
      ),
      (
        "character_desc_part.xml",
        "<specific_character id=\"a\"/>\n<specific_character id=\"b\"/>\n",
      ),
    ],
  )?;

  assert_eq!(partners.get_characters().len(), 2);
  assert!(partners.get_findings().is_empty(), "{:?}", partners.get_findings());

  Ok(())
}

#[test]
fn keeps_the_first_declaration_of_an_info_portion_even_when_it_opens_nothing() -> XrfResult {
  let partners: DialogPartners = read_partners(
    "info_duplicate",
    &[
      (
        "info_a.xml",
        r#"<game_information_portions><info_portion id="x"/></game_information_portions>"#,
      ),
      (
        "info_b.xml",
        r#"<game_information_portions><info_portion id="x"><dialog>d</dialog></info_portion></game_information_portions>"#,
      ),
    ],
  )?;

  assert!(partners.get_info_portions()["x"].get_dialogs().is_empty());
  assert_eq!(
    partners.get_info_portions()["x"].get_logical_path(),
    r"configs\gameplay\info_a.xml"
  );
  assert_eq!(partners.get_findings().len(), 1);
  assert!(partners.get_findings()[0].message.starts_with("Info portion 'x'"));

  Ok(())
}
