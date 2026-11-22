use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use uuid::Uuid;
use xrf_math::Vector3d;
use xrf_report::{CheckId, CheckReport};
use xrf_spawn::{
  AlifeObject, AlifeObjectHumanAbstract, AlifeObjectInherited, AlifeStalker, ClsId, GraphCrossTable, GraphHeader,
  GraphLevel, SpawnALifeSpawnsChunk, SpawnArtefactSpawnsChunk, SpawnFile, SpawnGraphsChunk, SpawnHeaderChunk,
  SpawnPacketHeader, SpawnPatrolsChunk,
};
use xrf_test_utils::GeneratedResources;

use super::dialogs_verifier::DialogsVerifier;
use super::verify_dialogs_result::GamedataDialogsVerificationResult;
use crate::project::gamedata_check::GamedataCheck;
use crate::project::gamedata_check_result::GamedataCheckResult;
use crate::project::gamedata_project::GamedataProject;
use crate::project::gamedata_project_read_options::GamedataProjectReadOptions;
use crate::project::gamedata_project_verify_options::GamedataProjectVerifyOptions;
use crate::project::gamedata_verification_status::GamedataVerificationStatus;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

const DIALOGS: &str = r#"<?xml version="1.0" encoding="windows-1251"?>
<game_dialogs>
  <dialog id="hello_dialog">
    <phrase_list>
      <phrase id="0"><text>hello_0</text><next>1</next></phrase>
      <phrase id="1"><text>hello_1</text><next>0</next></phrase>
    </phrase_list>
  </dialog>
  <dialog id="scripted_dialog">
    <init_func>dialogs.build</init_func>
  </dialog>
</game_dialogs>
"#;

const CHARACTERS: &str = r#"<xml>
  <specific_character id="novice" team_default="1">
    <class>novice</class>
    <start_dialog>hello_dialog</start_dialog>
    <actor_dialog>scripted_dialog</actor_dialog>
  </specific_character>
</xml>
"#;

const PROFILES: &str = r#"<xml><character id="novice"><class>novice</class></character></xml>"#;

const INFO_PORTIONS: &str = r#"<game_information_portions><info_portion id="known"><dialog>hello_dialog</dialog></info_portion></game_information_portions>"#;

const STRINGS: &str = r#"<string_table><string id="hello_0"><text>Hello.</text></string><string id="hello_1"><text>Bye.</text></string></string_table>"#;

/// A section spawning NPCs with the profile, which makes it one the engine has to resolve.
const SYSTEM: &str = "[novice_spawn]\ncharacter_profile = novice\n";

/// The `system.ltx` lists of a tree that names every file it holds.
const LISTS: &str = "[dialogs]\nfiles = dialogs\n\n[profiles]\nfiles = npc_profile\n\
                     specific_characters_files = character_desc_general\n\n[info_portions]\nfiles = info_portions\n";

/// The script the scripted dialog's `init_func` names a function of.
const SCRIPT: &str = "function build(dialog) end\n";

const LOGIC: &str =
  "[meet@talk]\nmeet_dialog = {+known} hello_dialog, nil\n\n[wounded@help]\nhelp_dialog = hello_dialog\n";

/// A game tree whose dialogs open and walk as the engine loads them, and whose partners offer only what it declares.
struct DialogTree {
  root: PathBuf,
}

impl DialogTree {
  fn new() -> Self {
    let unique: u64 = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
    let root: PathBuf = GeneratedResources::create_dir(&format!("gamedata_dialogs/fixture-{unique}")).unwrap();

    Self { root }
      .with("configs/system.ltx", SYSTEM)
      .with("configs/gameplay/dialogs.xml", DIALOGS)
      .with("configs/gameplay/character_desc_general.xml", CHARACTERS)
      .with("configs/gameplay/npc_profile.xml", PROFILES)
      .with("configs/gameplay/info_portions.xml", INFO_PORTIONS)
      .with("configs/text/eng/st_dialogs.xml", STRINGS)
      .with("configs/scripts/novice.ltx", LOGIC)
      .with("scripts/dialogs.script", SCRIPT)
  }

  fn with(self, relative: &str, contents: &str) -> Self {
    let path: PathBuf = self.root.join(relative);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();

    self
  }

  /// The tree with a root spawn placing one stalker per NPC.
  fn with_spawn(self, npcs: &[SpawnedNpc]) -> Self {
    self.write_spawn(npcs, &[]);
    self
  }

  /// The tree with a root spawn placing nothing, whose game graph holds the named levels, ids in their order.
  fn with_graph_levels(self, levels: &[&str]) -> Self {
    self.write_spawn(&[], levels);
    self
  }

  fn write_spawn(&self, npcs: &[SpawnedNpc], levels: &[&str]) {
    let guid: Uuid = Uuid::nil();
    let levels: Vec<GraphLevel> = levels
      .iter()
      .enumerate()
      .map(|(id, name)| GraphLevel {
        name: String::from(*name),
        offset: Vector3d::new(0.0, 0.0, 0.0),
        id: u8::try_from(id).unwrap(),
        section: format!("level{id}"),
        guid,
      })
      .collect();
    let levels_count: u8 = u8::try_from(levels.len()).unwrap();
    // The graph carries one cross table per level, empty for a level with no vertices.
    let cross_tables: Vec<GraphCrossTable> = levels
      .iter()
      .map(|_| GraphCrossTable {
        version: 10,
        nodes_count: 0,
        vertices_count: 0,
        level_guid: guid,
        game_guid: guid,
        cells: Vec::new(),
      })
      .collect();
    let spawn: SpawnFile = SpawnFile {
      header: SpawnHeaderChunk {
        version: 10,
        guid,
        graph_guid: guid,
        objects_count: u32::try_from(npcs.len()).unwrap(),
        levels_count: u32::from(levels_count),
      },
      alife_spawn: SpawnALifeSpawnsChunk {
        objects: npcs.iter().enumerate().map(|(id, npc)| npc.build(id)).collect(),
      },
      artefact_spawn: SpawnArtefactSpawnsChunk { nodes: Vec::new() },
      patrols: SpawnPatrolsChunk { patrols: Vec::new() },
      graphs: SpawnGraphsChunk {
        header: GraphHeader {
          version: 10,
          vertices_count: 0,
          edges_count: 0,
          points_count: 0,
          guid,
          levels_count,
        },
        levels,
        vertices: Vec::new(),
        edges: Vec::new(),
        points: Vec::new(),
        cross_tables,
      },
    };

    spawn
      .write_to_path(&self.root.join("spawns").join("all.spawn"))
      .unwrap();
  }

  fn verify(&self) -> GamedataDialogsVerificationResult {
    let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
      root: self.root.clone(),
      ..Default::default()
    })
    .unwrap();

    DialogsVerifier::new(&project, &GamedataProjectVerifyOptions::default())
      .verify()
      .unwrap()
  }
}

impl Drop for DialogTree {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

/// A stalker a root spawn places: the character or profile it is spawned as, and the logic its custom data carries.
struct SpawnedNpc {
  name: &'static str,
  specific_character: &'static str,
  profile: &'static str,
  custom_data: &'static str,
}

impl SpawnedNpc {
  /// A stalker spawned with the fixture's resolvable profile and no logic of its own.
  const NOVICE: Self = Self {
    name: "esc_novice",
    specific_character: "",
    profile: "novice",
    custom_data: "",
  };

  fn build(&self, id: usize) -> AlifeObject {
    let mut stalker: AlifeStalker = AlifeStalker::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let human: &mut AlifeObjectHumanAbstract = &mut stalker.base.base;

    human.trader.specific_character = String::from(self.specific_character);
    human.trader.character_profile = String::from(self.profile);

    let mut inherited: AlifeObjectInherited = AlifeObjectInherited::SeStalker(Box::new(stalker));

    if let Some(base) = inherited.get_abstract_mut() {
      base.custom_data = String::from(self.custom_data);
    }

    AlifeObject {
      header: SpawnPacketHeader {
        id: u16::try_from(id).unwrap(),
        net_action: 1,
        section: String::from("stalker"),
        name: String::from(self.name),
        script_game_id: 2,
        script_rp: 3,
        position: Vector3d::new(1.0, 2.0, 3.0),
        direction: Vector3d::new(0.0, 0.0, 0.0),
        respawn_time: 0,
        parent_id: u16::MAX,
        phantom_id: u16::MAX,
        // The engine reads a packet's version only under its version flag.
        script_flags: 33,
        version: 128,
        game_type: 1,
        script_version: 10,
        client_data: Vec::new(),
        spawn_id: u16::try_from(id).unwrap(),
      },
      clsid: ClsId::AiStlS,
      inherited,
    }
  }
}

fn rules(result: &GamedataDialogsVerificationResult) -> Vec<&str> {
  result
    .get_findings()
    .iter()
    .map(|finding| finding.rule_id().as_str())
    .collect()
}

fn messages(result: &GamedataDialogsVerificationResult) -> Vec<&str> {
  result.get_findings().iter().map(|finding| finding.message()).collect()
}

/// The check's verdict in a strict run, which fails it on a warning too.
fn judge_strictly(result: &GamedataDialogsVerificationResult) -> GamedataVerificationStatus {
  CheckReport::new(
    CheckId::of_static("dialogs"),
    result.get_status(),
    None,
    result.get_findings().to_vec(),
  )
  .into_strict()
  .status()
}

#[test]
fn passes_a_tree_whose_dialogs_and_partners_hold_together() {
  let result: GamedataDialogsVerificationResult = DialogTree::new().verify();

  assert_eq!(messages(&result), Vec::<&str>::new());
  assert_eq!(judge_strictly(&result), GamedataVerificationStatus::Passed);
  // A start and an actor dialog, an info portion's, the meet condlist's one dialog past `nil`, the heal dialog, the
  // profile a section spawns with, and the scripted dialog's `init_func`.
  assert_eq!(
    result.get_summary(),
    "2/2 dialogs valid; 1/1 profiles resolve; 7/7 references declared"
  );
}

#[test]
fn skips_a_tree_without_dialog_files() {
  let tree: DialogTree = DialogTree::new();

  fs::remove_file(tree.root.join("configs/gameplay/dialogs.xml")).unwrap();

  let result: GamedataDialogsVerificationResult = tree.verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Skipped);
  assert_eq!(result.get_summary(), "No dialog files found");
}

#[test]
fn fails_an_offer_or_start_dialog_no_file_declares() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with(
      "configs/gameplay/character_desc_general.xml",
      &CHARACTERS.replace("<actor_dialog>scripted_dialog", "<actor_dialog>gone_dialog"),
    )
    .with(
      "configs/gameplay/info_portions.xml",
      &INFO_PORTIONS.replace("hello_dialog", "lost_dialog"),
    )
    .with(
      "configs/scripts/novice.ltx",
      "[meet]\nmeet_dialog = absent_dialog\n\n[wounded]\nhelp_start_dialog = absent_dialog\n",
    )
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_eq!(
    rules(&result),
    [
      "dialog.offer-missing",
      "dialog.offer-missing",
      "dialog.logic-missing",
      "dialog.logic-missing"
    ]
  );
  assert_eq!(
    messages(&result),
    [
      "Character 'novice' offers actor dialog 'gone_dialog', which no dialog file declares",
      "Info portion 'known' opens dialog 'lost_dialog', which no dialog file declares",
      "[meet] meet_dialog names dialog 'absent_dialog', which no dialog file declares",
      "[wounded] help_start_dialog names dialog 'absent_dialog', which no dialog file declares",
    ]
  );
  assert_eq!(result.missing_references_count, 4);
}

/// `help_dialog` only marks which dialog heals the NPC, so the engine goes on without it.
#[test]
fn reports_a_heal_dialog_no_file_declares_and_fails_it_only_under_strict() {
  let tree: DialogTree =
    DialogTree::new().with("configs/scripts/novice.ltx", "[wounded]\nhelp_dialog = absent_dialog\n");

  let lenient: GamedataDialogsVerificationResult = tree.verify();

  assert_eq!(rules(&lenient), ["dialog.heal-missing"]);
  assert_eq!(lenient.get_status(), GamedataVerificationStatus::Passed);
  assert!(
    lenient.get_summary().ends_with(", 1 warnings"),
    "{}",
    lenient.get_summary()
  );
  assert_eq!(judge_strictly(&lenient), GamedataVerificationStatus::Failed);
}

/// A profile stops the engine only once an NPC is spawned with it, which configs do not tell, so it is reported; the
/// flags a character cannot carry together stop the engine as soon as it loads the characters.
#[test]
fn reports_a_profile_no_npc_can_be_spawned_with_and_fails_conflicting_character_flags() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with(
      "configs/system.ltx",
      "[pinned_spawn]\ncharacter_profile = pinned\n\n[ghost_spawn]\ncharacter_profile = ghost\n",
    )
    .with(
      "configs/gameplay/npc_profile.xml",
      r#"<xml>
  <character id="pinned"><specific_character>nobody</specific_character></character>
  <character id="loner"><class>loner</class></character>
</xml>"#,
    )
    .with(
      "configs/gameplay/character_desc_loner.xml",
      r#"<xml><specific_character id="lone"><class>loner</class></specific_character>
<specific_character id="both" no_random="1" team_default="1"><class>loner</class></specific_character></xml>"#,
    )
    .verify();

  assert_eq!(
    messages(&result),
    [
      "Character 'both' is marked both no_random and team_default, which the engine asserts on loading it",
      "Profile 'loner' picks by class 'loner', which no team_default character without no_random has",
      "Profile 'pinned' pins character 'nobody', which no character file declares, and section [pinned_spawn] names \
       it as its character_profile",
      "Section [ghost_spawn] character_profile names profile 'ghost', which no profile file declares",
    ]
  );
  assert_eq!(rules(&result)[0], "dialog.character-flags");
  assert_eq!(result.unresolved_profiles_count, 2);
  assert_eq!(result.count_failures(), 1);
  assert_eq!(result.get_findings().len(), 4);
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

/// The engine looks a function up in the namespace its script file fills, or in a table xrf-engine fills through
/// `extern`, and throws on one it cannot find.
#[test]
fn fails_a_script_function_a_dialog_names_and_no_script_defines() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with(
      "configs/gameplay/dialogs_scripted.xml",
      r#"<game_dialogs><dialog id="scripted"><precondition>dialogs.gone</precondition><phrase_list>
  <phrase id="0"><text>hello_0</text><precondition>dialogs_zaton.ignored</precondition><next>1</next></phrase>
  <phrase id="1"><text>hello_1</text><action>dialogs_zaton.registered</action><action>dialogs.build</action>
    <script_text>dialogs_zaton.absent</script_text></phrase>
  <phrase id="2"><text>hello_1</text><action>dialogs.unreached</action></phrase>
</phrase_list><init_func>dialogs.not_read</init_func></dialog></game_dialogs>"#,
    )
    .with(
      "declarations/dialogs/zaton.script",
      "extern(\"dialogs_zaton.registered\", function() end)\n",
    )
    .verify();

  assert_eq!(rules(&result)[0], "dialog.functor-missing");
  assert_eq!(
    messages(&result),
    [
      "Dialog 'scripted' phrase '1' script_text names script function 'dialogs_zaton.absent', which no script defines",
      "Dialog 'scripted' precondition names script function 'dialogs.gone', which no script defines",
      // Phrase `2` is never reached, phrase `0`'s precondition never read, and `init_func` read only without phrases.
      "Dialog 'scripted' phrases '2' are not reached from phrase '0', so the engine never loads them",
    ]
  );
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

/// A tree holding no scripts, such as a mod relying on the game's, tells nothing of what the scripts define.
#[test]
fn does_not_judge_functions_in_a_tree_holding_no_scripts() {
  let tree: DialogTree = DialogTree::new();

  fs::remove_file(tree.root.join("scripts/dialogs.script")).unwrap();

  let result: GamedataDialogsVerificationResult = tree
    .with(
      "configs/gameplay/dialogs_scripted.xml",
      r#"<game_dialogs><dialog id="d"><precondition>dialogs.gone</precondition><phrase_list>
  <phrase id="0"><text>hello_0</text></phrase></phrase_list></dialog></game_dialogs>"#,
    )
    .verify();

  assert_eq!(messages(&result), Vec::<&str>::new());
}

/// A script that does not parse hides what it defines, and the scripts check reports it, so its namespace is not
/// judged here.
#[test]
fn does_not_judge_a_function_of_a_script_that_does_not_parse() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with("scripts/dialogs.script", "function build(dialog)\n")
    .verify();

  assert_eq!(messages(&result), Vec::<&str>::new());
}

/// A squad section and the NPC sections it places: a stalker with an unresolvable profile, and a dog, whose class reads
/// no profile.
const SQUADS: &str = "[loner_squad]\nnpc = loner_npc, dog_npc\n\n[loner_npc]\nclass = AI_STL_S\ncharacter_profile = \
                      loner\n\n[dog_npc]\nclass = SM_DOG_S\ncharacter_profile = novice\n";

/// Profiles with one no character resolves.
const LONER_PROFILES: &str = r#"<xml><character id="novice"><class>novice</class></character><character id="loner"><class>loner</class></character></xml>"#;

/// A squad nothing spawns leaves its profiles to warnings; one a smart terrain respawns fails them as it places them.
#[test]
fn fails_a_profile_a_squad_a_smart_terrain_spawns_places_and_no_character_resolves() {
  let tree: DialogTree = DialogTree::new()
    .with("configs/system.ltx", &format!("{SYSTEM}\n{SQUADS}"))
    .with("configs/gameplay/npc_profile.xml", LONER_PROFILES);

  assert_eq!(rules(&tree.verify()), ["dialog.profile-unresolved"]);

  let result: GamedataDialogsVerificationResult = tree
    .with(
      "configs/scripts/smart.ltx",
      "[respawn@base]\nspawn_squads = loner_squad\n",
    )
    .verify();

  assert_eq!(
    messages(&result),
    [
      "Squad [loner_squad] places NPC section [loner_npc] with a profile no character resolves: Profile 'loner' picks \
       by class 'loner', which no team_default character without no_random has"
    ]
  );
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

/// `sim_board:fill_start_position` creates its squads with a new game, and `create_squad` the one its condlist names.
/// The start positions are read for the levels of the game graph in id order, up to the first with none, so neither a
/// test level the graph does not hold nor a level after `pripyat`, which has none, starts a squad.
#[test]
fn reads_the_squads_a_new_game_starts_and_a_condlist_creates() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with_graph_levels(&["zaton", "pripyat", "jupiter"])
    .with(
      "configs/system.ltx",
      &format!(
        "{SYSTEM}\n[start_squad]\nnpc = start_npc\n\n[start_npc]\nclass = AI_STL_S\ncharacter_profile = gone_start\n\n\
         [effect_squad]\nnpc_random = effect_npc\n\n[effect_npc]\nclass = AI_STL_S\n"
      ),
    )
    .with(
      "configs/misc/simulation.ltx",
      "[start_position_zaton]\nstart_squad = zat_smart\n\n[start_position_zaton_test]\ngone_test_squad = zat_smart\n\n\
       [start_position_jupiter]\ngone_jupiter_squad = jup_smart\n",
    )
    .with(
      "configs/scripts/effects.ltx",
      "[sr_idle@spawn]\non_info = %=create_squad(effect_squad:zat_smart)%\n",
    )
    .verify();

  assert_eq!(
    messages(&result),
    [
      // An NPC section naming no profile is spawned with `default`.
      "Squad [effect_squad] places NPC section [effect_npc] with profile 'default', which no profile file declares",
      "Squad [start_squad] places NPC section [start_npc] with profile 'gone_start', which no profile file declares",
    ]
  );
}

/// `alife():create` asserts on a section no config declares: a squad a config spawns fails, as does an NPC section a
/// spawned squad lists. One only a squad nothing spawns lists is a warning, since a script may still create that squad.
#[test]
fn reports_squad_and_npc_sections_no_config_declares() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with(
      "configs/system.ltx",
      &format!(
        "{SYSTEM}\n[squad_base]\nclass = ON_OFF_S\n\n[spawned_squad]:squad_base\nnpc = gone_npc\n\n\
         [scripted_squad]:squad_base\nnpc_random = gone_scripted_npc, gone_npc\n"
      ),
    )
    .with(
      "configs/scripts/smart.ltx",
      "[respawn@base]\nspawn_squads = spawned_squad, gone_squad\n",
    )
    .verify();

  assert_eq!(
    messages(&result),
    [
      "Squad [gone_squad] is spawned here, but no section declares it",
      "Squad [scripted_squad] lists NPC section [gone_scripted_npc], which no section declares; no config spawns the \
       squad",
      "Squad [spawned_squad] lists NPC section [gone_npc], which no section declares",
    ]
  );
  assert_eq!(rules(&result), ["dialog.squad-missing"; 3]);
  assert_eq!(result.count_failures(), 2);
}

/// A new game reads every NPC of the root spawn and resolves its character there, so one it cannot resolve stops it.
#[test]
fn fails_an_npc_spawned_as_a_character_or_with_a_profile_nothing_declares() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with_spawn(&[
      SpawnedNpc::NOVICE,
      SpawnedNpc {
        name: "esc_stranger",
        specific_character: "nobody",
        ..SpawnedNpc::NOVICE
      },
      SpawnedNpc {
        name: "esc_ghost",
        profile: "ghost",
        ..SpawnedNpc::NOVICE
      },
    ])
    .verify();

  assert_eq!(rules(&result), ["dialog.spawn-unresolved", "dialog.spawn-unresolved"]);
  assert_eq!(
    messages(&result),
    [
      "NPC 'esc_ghost' of section [stalker] is spawned with profile 'ghost', which no profile file declares",
      "NPC 'esc_stranger' of section [stalker] is spawned as character 'nobody', which no character file declares",
    ]
  );
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

/// A profile nothing spawns is only reported; one a spawned NPC uses fails, under that NPC's finding alone.
#[test]
fn fails_an_npc_spawned_with_a_profile_no_character_resolves_instead_of_reporting_the_profile() {
  let tree: DialogTree = DialogTree::new().with(
    "configs/gameplay/npc_profile.xml",
    r#"<xml><character id="novice"><class>novice</class></character>
<character id="loner"><class>loner</class></character></xml>"#,
  );

  assert_eq!(rules(&tree.verify()), ["dialog.profile-unresolved"]);

  let result: GamedataDialogsVerificationResult = tree
    .with_spawn(&[SpawnedNpc {
      name: "esc_loner",
      profile: "loner",
      ..SpawnedNpc::NOVICE
    }])
    .verify();

  assert_eq!(
    messages(&result),
    [
      "NPC 'esc_loner' of section [stalker] is spawned with a profile no character resolves: Profile 'loner' picks \
       by class 'loner', which no team_default character without no_random has"
    ]
  );
  assert_eq!(result.unresolved_profiles_count, 1);
}

#[test]
fn fails_a_start_dialog_the_logic_of_an_npc_custom_data_names_and_nothing_declares() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with_spawn(&[SpawnedNpc {
      custom_data: "[logic]\nactive = walker@a\nmeet = meet@a\n\n[meet@a]\nmeet_dialog = absent_dialog\n",
      ..SpawnedNpc::NOVICE
    }])
    .verify();

  assert_eq!(rules(&result), ["dialog.logic-missing"]);
  assert_eq!(
    messages(&result),
    ["NPC 'esc_novice' [meet@a] meet_dialog names dialog 'absent_dialog', which no dialog file declares"]
  );
}

/// The engine loads only the partner files `system.ltx` lists, so an unlisted one declares nothing, whatever its name.
#[test]
fn reads_only_the_partner_files_system_ltx_lists() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with("configs/system.ltx", &format!("{SYSTEM}\n{LISTS}"))
    .with(
      "configs/gameplay/character_desc_unlisted.xml",
      r#"<xml><specific_character id="stray"><actor_dialog>gone_dialog</actor_dialog></specific_character></xml>"#,
    )
    .verify();

  assert_eq!(rules(&result), ["dialog.file-unlisted"]);
  assert_eq!(
    messages(&result),
    ["Partner file is not listed in system.ltx, so the engine never loads what it declares"]
  );
  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
}

/// The engine loads only the dialog files `[dialogs] files` lists, so a dialog only an unlisted file declares is one
/// it never has.
#[test]
fn reads_only_the_dialog_files_system_ltx_lists() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with(
      "configs/system.ltx",
      &format!("{SYSTEM}\n{}", LISTS.replace("files = dialogs", "files = dialogs_main")),
    )
    .with("configs/gameplay/dialogs_main.xml", DIALOGS)
    .with(
      "configs/gameplay/dialogs_stray.xml",
      r#"<game_dialogs><dialog id="stray_dialog"><phrase_list><phrase id="0"/></phrase_list></dialog></game_dialogs>"#,
    )
    .with(
      "configs/gameplay/info_portions.xml",
      &INFO_PORTIONS.replace("hello_dialog", "stray_dialog"),
    )
    .verify();

  assert_eq!(
    messages(&result),
    [
      // The fixture's own `dialogs.xml` and the stray file, both left out of the list.
      "Dialog file is not listed in [dialogs] files, so the engine never loads its dialogs",
      "Dialog file is not listed in [dialogs] files, so the engine never loads its dialogs",
      "Info portion 'known' opens dialog 'stray_dialog', which no dialog file declares",
    ]
  );
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

/// The engine asserts on a listed dialog, character or profile file it cannot find, and skips an info portion file.
#[test]
fn fails_a_listed_file_that_is_missing_and_reports_a_missing_info_portion_file() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with(
      "configs/system.ltx",
      &format!(
        "{SYSTEM}\n{}",
        LISTS
          .replace("files = dialogs", "files = dialogs, dialogs_gone")
          .replace("character_desc_general", "character_desc_general, character_desc_gone")
          .replace("files = info_portions", "files = info_portions, info_gone")
      ),
    )
    .verify();

  assert_eq!(
    rules(&result),
    [
      "dialog.file-missing",
      "dialog.file-missing",
      "dialog.partner-unreadable"
    ]
  );
  assert_eq!(
    messages(&result),
    [
      "Partner file is listed in system.ltx, but the tree does not hold it, which the engine asserts on",
      "Dialog file is listed in [dialogs] files, but the tree does not hold it, which the engine asserts on",
      "Info portion file is listed in system.ltx, but the tree does not hold it, so the engine skips it",
    ]
  );
  assert!(
    result.get_findings()[0]
      .subject()
      .is_some_and(|subject| subject.ends_with("character_desc_gone.xml")),
    "{:?}",
    result.get_findings()
  );
  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
}

/// The engine's id index keeps the first declaration of an id and skips the rest with only a log line.
#[test]
fn reports_an_id_declared_twice() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with(
      "configs/gameplay/npc_profile.xml",
      r#"<xml><character id="novice"><class>novice</class></character>
<character id="novice"><class>loner</class></character></xml>"#,
    )
    .verify();

  assert_eq!(rules(&result), ["dialog.duplicate"]);
  assert_eq!(
    messages(&result),
    ["Profile 'novice' is declared again, and the first declaration is the one the engine reads"]
  );
  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
}

#[test]
fn fails_a_dialog_the_engine_cannot_open_or_walk() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with(
      "configs/gameplay/dialogs_broken.xml",
      r#"<game_dialogs>
  <dialog id="no_entry"><phrase_list><phrase id="1"><text>hello_0</text></phrase></phrase_list></dialog>
  <dialog id="no_list"><phrase id="0"><text>hello_0</text></phrase></dialog>
  <dialog id="dangling"><phrase_list>
    <phrase id="0"><text>hello_0</text><next>1</next><next>7</next></phrase>
    <phrase id="1"><text>hello_1</text></phrase>
    <phrase id="5"><text>hello_1</text><next>8</next></phrase>
  </phrase_list></dialog>
</game_dialogs>"#,
    )
    .verify();

  assert_eq!(
    messages(&result),
    [
      "Dialog 'no_entry' lists no phrase '0', which the engine opens it with",
      "Dialog 'no_list' has no phrase list and no init_func to build one",
      "Dialog 'dangling' phrase '0' leads to phrase '7', which the dialog does not list",
      // Its own broken `next` is never followed, since the engine never reaches the phrase.
      "Dialog 'dangling' phrases '5' are not reached from phrase '0', so the engine never loads them",
    ]
  );
  assert_eq!(
    rules(&result),
    [
      "dialog.phrase-entry",
      "dialog.phrase-entry",
      "dialog.phrase-next",
      "dialog.phrase-unreachable"
    ]
  );
  assert_eq!(result.invalid_dialogs_count, 3);
  assert_eq!(result.checked_dialogs_count, 5);
}

/// The engine reads the first declaration of an id, so a later one is reported but not judged.
#[test]
fn judges_only_the_declaration_the_engine_reads() {
  let result: GamedataDialogsVerificationResult = DialogTree::new()
    .with(
      "configs/gameplay/dialogs_overlay.xml",
      r#"<game_dialogs><dialog id="hello_dialog"><phrase_list><phrase id="1"/></phrase_list></dialog></game_dialogs>"#,
    )
    .verify();

  assert_eq!(
    messages(&result),
    ["Dialog 'hello_dialog' is declared again, and the first declaration is the one the engine reads"]
  );
  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
  assert_eq!(result.checked_dialogs_count, 2);
}

#[test]
fn reports_a_line_the_text_does_not_define_and_fails_it_only_under_strict() {
  let tree: DialogTree = DialogTree::new().with(
    "configs/text/rus/st_dialogs.xml",
    r#"<string_table><string id="hello_0"><text>Privet.</text></string></string_table>"#,
  );

  let lenient: GamedataDialogsVerificationResult = tree.verify();

  assert_eq!(
    messages(&lenient),
    ["Dialog 'hello_dialog' phrase '1' text 'hello_1' has no line in rus"]
  );
  assert_eq!(rules(&lenient), ["dialog.text-missing"]);
  assert_eq!(lenient.get_status(), GamedataVerificationStatus::Passed);
  assert_eq!(judge_strictly(&lenient), GamedataVerificationStatus::Failed);
}
