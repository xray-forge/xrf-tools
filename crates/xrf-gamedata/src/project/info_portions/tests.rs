use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use xrf_report::Finding;
use xrf_test_utils::GeneratedResources;

use super::info_portions_verifier::InfoPortionsVerifier;
use super::verify_info_portions_result::GamedataInfoPortionsVerificationResult;
use crate::project::gamedata_check::GamedataCheck;
use crate::project::gamedata_check_result::GamedataCheckResult;
use crate::project::gamedata_project::GamedataProject;
use crate::project::gamedata_project_read_options::GamedataProjectReadOptions;
use crate::project::gamedata_project_verify_options::GamedataProjectVerifyOptions;
use crate::project::gamedata_verification_status::GamedataVerificationStatus;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

/// A game tree written file by file under a root of its own, since tests run in parallel.
struct InfoPortionTree {
  root: PathBuf,
}

impl InfoPortionTree {
  fn new() -> Self {
    let unique: u64 = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
    let root: PathBuf = GeneratedResources::create_dir(&format!("gamedata_info_portions/fixture-{unique}")).unwrap();

    Self { root }.with("configs/system.ltx", "[actor]\nclass = O_ACTOR\n")
  }

  fn with(self, relative: &str, contents: &str) -> Self {
    let path: PathBuf = self.root.join(relative);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();

    self
  }

  fn verify(&self) -> GamedataInfoPortionsVerificationResult {
    let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
      root: self.root.clone(),
      ..Default::default()
    })
    .unwrap();

    InfoPortionsVerifier::new(&project, &GamedataProjectVerifyOptions::default())
      .verify()
      .unwrap()
  }
}

impl Drop for InfoPortionTree {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

/// The logic reading every portion the tests follow, one of them behind a zone, as `on_actor_in_zone` writes it, one
/// misspelt and one a dialog file the lists leave out gives.
const READER: &str = "[sr_idle@wait]\non_info = {+given_info +dialog_info +built_info_2 +data_info} sr_idle@next\n\
                      on_info2 = {-scripted_info -bone_info -passed_info} sr_idle@next\n\
                      on_actor_in_zone = zone | {+gone_info} sr_idle@next\n\
                      on_info3 = {+table_info -taken_info} sr_idle@next\n\
                      on_info4 = {+layer_info +given_inf0 +unlisted_info} sr_idle@next\n\
                      on_info5 = {+searched_info +default_searched_info +scene_info} sr_idle@next\n";

/// An NPC's logic naming its known info section, and another's relying on the default one: the actor takes both on
/// searching the body.
const KNOWN_INFO: &str = "[logic]\nknown_info = known_info@body\n\n[known_info@body]\nsearched_info\n";
const DEFAULT_KNOWN_INFO: &str = "[logic]\nactive = walker@idle\n\n[known_info]\ndefault_searched_info\n";

/// The debug scenes a developer menu jumps into, the first starting with the portion its quest reads.
const SCENES: &str = "[zaton]\nscene_one = scene_info | walk_path | look_path | wpn_ak74\n";

/// Logic giving portions: in a plain effect, in the second condlist of a field holding two, and by passing one to a
/// condlist function.
const GIVER: &str = "[sr_idle@give]\non_info = sr_idle@nil %+given_info -taken_info%\n\
                     hit_on_bone = 4 | ph_door@open %+other_info% | 2 | ph_door@open %+bone_info%\n\
                     on_info2 = {=surge_started} | %=create_random_infop(passed_info:other_info)%\n";

/// The `system.ltx` listing the dialog file that gives `dialog_info`, and not the one giving `unlisted_info`.
const LISTED_SYSTEM: &str = "[dialogs]\nfiles = dialogs\n";

/// A dialog whose entry phrase gives a portion.
const GIVING_DIALOGS: &str = r#"<game_dialogs><dialog id="d"><phrase_list><phrase id="0"><give_info>dialog_info</give_info></phrase></phrase_list></dialog></game_dialogs>"#;

/// A dialog in a file the lists leave out, giving a portion the engine never hands over.
const UNLISTED_DIALOGS: &str = r#"<game_dialogs><dialog id="u"><phrase_list><phrase id="0"><give_info>unlisted_info</give_info></phrase></phrase_list></dialog></game_dialogs>"#;

/// A script mirroring one name as a constant, keeping one as data, passing one to a call and building names onto a
/// family's prefix and onto a word.
const QUEST_SCRIPT: &str = "local names = { table_info = \"table_info\" }\nlocal orders = { ordered = \"data_info\" }\n\
                            function give(index)\n  db.actor:give_info_portion(\"scripted_info\")\n  \
                            db.actor:give_info_portion(\"built_info_\" .. index)\n  print(\"layer_\" .. index)\nend\n";

fn messages(result: &GamedataInfoPortionsVerificationResult) -> Vec<&str> {
  result.get_findings().iter().map(Finding::message).collect()
}

/// A portion is given by a condlist effect in any condlist of a field, a condlist function receiving it, a dialog's
/// `give_info` in a listed file, an NPC's known info section, a debug scene starting with it, or a script passing its
/// name to a call, keeping it in a table as data or building it onto a prefix; one only a table of constants mirrors,
/// one only taken away, or one only an unlisted dialog file gives, is still never given.
#[test]
fn reports_a_portion_read_that_nothing_gives() {
  let result: GamedataInfoPortionsVerificationResult = InfoPortionTree::new()
    .with("configs/system.ltx", LISTED_SYSTEM)
    .with("configs/scripts/reader.ltx", READER)
    .with("configs/scripts/giver.ltx", GIVER)
    .with("configs/gameplay/dialogs.xml", GIVING_DIALOGS)
    .with("configs/gameplay/dialogs_unlisted.xml", UNLISTED_DIALOGS)
    .with("configs/scripts/body.ltx", KNOWN_INFO)
    .with("configs/scripts/default_body.ltx", DEFAULT_KNOWN_INFO)
    .with("configs/scripts/scenes.ltx", SCENES)
    .with("scripts/quest.script", QUEST_SCRIPT)
    .verify();

  assert_eq!(
    messages(&result),
    [
      // One character from `given_info`, which a condlist gives.
      "Info portion 'given_inf0' is read by [sr_idle@wait] on_info4, but no condlist, dialog or spawned document \
       gives it and no script names it; did you mean 'given_info'?",
      "Info portion 'gone_info' is read by [sr_idle@wait] on_actor_in_zone, but no condlist, dialog or spawned \
       document gives it and no script names it",
      // `layer_` is a word, not a family of names, so building onto it gives nothing.
      "Info portion 'layer_info' is read by [sr_idle@wait] on_info4, but no condlist, dialog or spawned document \
       gives it and no script names it",
      "Info portion 'table_info' is read by [sr_idle@wait] on_info3, but no condlist, dialog or spawned document \
       gives it and no script names it",
      "Info portion 'taken_info' is read by [sr_idle@wait] on_info3, but no condlist, dialog or spawned document \
       gives it and no script names it",
      "Info portion 'unlisted_info' is read by [sr_idle@wait] on_info4, but no condlist, dialog or spawned document \
       gives it and no script names it; dialog 'u' phrase 0 of configs\\gameplay\\dialogs_unlisted.xml would, but \
       system.ltx does not list its file",
    ]
  );
  // The engine never stops on a spelling, so every finding is a warning and a lenient run passes.
  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
}

/// A portion given and never read is reported where it is first given, unless a script reads it through a reader call
/// or names it where what it does is unknown, such as a table keyed by it, or its declaration opens a dialog or takes
/// another portion away. A script
/// reading a portion does not give it, so one only a reader call names is still read and never given.
#[test]
fn reports_a_portion_given_that_nothing_reads() {
  let result: GamedataInfoPortionsVerificationResult = InfoPortionTree::new()
    .with(
      "configs/scripts/giver.ltx",
      "[sr_idle@give]\non_info = sr_idle@nil %+dead_state +script_read_state +opening_state +disabling_state \
       +handed_state +keyed_state%\non_info2 = {+script_only_read} sr_idle@nil\n",
    )
    .with(
      "configs/gameplay/info_portions.xml",
      r#"<game_information_portions>
  <info_portion id="opening_state"><dialog>d</dialog></info_portion>
  <info_portion id="disabling_state"><disable>dead_state_old</disable></info_portion>
</game_information_portions>"#,
    )
    .with(
      "scripts/quest.script",
      "function check()\n  return has_alife_info(\"script_read_state\") and has_alife_info(\"script_only_read\")\nend\n\
       function hand()\n  handlers.store(\"handed_state\")\nend\nlocal supplies = { keyed_state = { ammo = 2 } }\n",
    )
    .verify();

  assert_eq!(
    messages(&result),
    [
      "Info portion 'script_only_read' is read by [sr_idle@give] on_info2, but no condlist, dialog or spawned document \
       gives it and no script names it",
      "Info portion 'dead_state' is given by [sr_idle@give] on_info, but no condlist, dialog or script reads it and \
       holding it does nothing",
    ]
  );
  assert_eq!(
    result
      .get_findings()
      .iter()
      .map(|finding| finding.rule_id().as_str())
      .collect::<Vec<&str>>(),
    ["info-portion.read-only", "info-portion.unread"]
  );
}

/// A tree naming no info portion, such as a mod touching only meshes, is not judged.
#[test]
fn skips_a_tree_naming_no_info_portion() {
  let result: GamedataInfoPortionsVerificationResult = InfoPortionTree::new().verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Skipped);
}

/// Scripts give portions too, so a tree holding none, such as a mod relying on the game's scripts, is not judged.
#[test]
fn skips_a_tree_holding_no_scripts() {
  let result: GamedataInfoPortionsVerificationResult = InfoPortionTree::new()
    .with("configs/scripts/reader.ltx", READER)
    .verify();

  assert_eq!(messages(&result), Vec::<&str>::new());
  assert_eq!(result.get_status(), GamedataVerificationStatus::Skipped);
  assert_eq!(
    result.get_summary(),
    "16 info portions named, but no scripts, so which are never given is unknown"
  );
}
