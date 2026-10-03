use std::fs;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use xrf_error::XrfResult;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayMountMode, XrayRoots, XrayVfs};

use crate::partner::dialog_character::DialogCharacter;
use crate::partner::dialog_offer::DialogOffer;
use crate::partner::dialog_partners::DialogPartners;
use crate::partner::dialog_profile::DialogProfile;
use crate::project::mode::DialogProjectMode;

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

/// Lay out a gameplay folder and read its partners.
fn read_partners(name: &str, files: &[(&str, &str)]) -> XrfResult<DialogPartners> {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("partner/{name}"));
  let gameplay: PathBuf = root.join("configs").join("gameplay");

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&gameplay)?;
  fs::write(gameplay.join("character_dialogs.xml"), CHARACTER_DIALOGS)?;

  for (file, content) in files {
    fs::write(gameplay.join(file), content)?;
  }

  let partners: DialogPartners = DialogPartners::from_vfs(&open(&root)?, DialogProjectMode::DIALOGS_PREFIX)?;

  fs::remove_dir_all(root)?;

  Ok(partners)
}

fn open(root: &Path) -> XrfResult<XrayVfs> {
  XrayRoots::one(root.to_path_buf(), XrayMountMode::Directory).open()
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
fn lists_start_then_actor_then_info_portion_dialogs_with_every_way_each_is_offered() -> XrfResult {
  let partners: DialogPartners = read_shipped("offers")?;
  let offers: IndexMap<String, Vec<DialogOffer>> = partners.list_offers(profile(&partners, "snag"));
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
  assert!(partners.get_info_dialogs()["no_dialogs"].is_empty());

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

  assert!(partners.get_info_dialogs()["x"].is_empty());
  assert_eq!(partners.get_findings().len(), 1);
  assert!(partners.get_findings()[0].message.starts_with("Info portion 'x'"));

  Ok(())
}
