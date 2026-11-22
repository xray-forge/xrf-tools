use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use xrf_dialog::{Dialog, DialogPartners};
use xrf_ltx::Ltx;
use xrf_spawn::{AlifeObject, AlifeObjectTraderAbstract, SpawnFile};

use crate::project::dialogs::dialog_logic_reference::DialogLogicReference;
use crate::project::dialogs::dialogs_verifier::DialogsVerifier;
use crate::project::dialogs::verify_dialogs_result::GamedataDialogsVerificationResult;
use crate::project::gamedata_verification_rule::GamedataVerificationRule;

/// Who the spawn files place: the character each NPC is spawned as, and the start dialogs its custom data's logic names.
impl DialogsVerifier<'_> {
  /// Every NPC the spawn files place resolves to a character, and every dialog the logic its custom data carries names
  /// is declared.
  ///
  /// The game reads every object of the spawn when a new game starts, and `CSE_ALifeTraderAbstract::STATE_Read` resolves
  /// its specific character there, asserting on one no file declares, on a profile no file declares, and on a class
  /// without a `team_default` character. Answers the profiles the NPCs are spawned with, whose findings these are.
  pub(super) fn verify_spawns(
    &self,
    partners: &DialogPartners,
    declared: &HashMap<&str, &Dialog>,
    result: &mut GamedataDialogsVerificationResult,
  ) -> HashSet<String> {
    let mut spawned: HashSet<String> = HashSet::new();

    for spawn_path in &self.project.spawn_paths() {
      if self.options.job.is_cancelled() {
        break;
      }

      // A spawn file that does not read is reported by the spawns check, not this one.
      let Ok(spawn_file) = self.project.read_parsed(spawn_path, SpawnFile::read_from_chunk) else {
        continue;
      };
      let path: PathBuf = self.report_path(spawn_path);

      for object in &spawn_file.alife_spawn.objects {
        spawned.extend(Self::verify_spawned_trader(partners, object, &path, result));
        self.verify_spawned_logic(declared, object, &path, result);
      }
    }

    spawned
  }

  /// An object with a trading part resolves as `CSE_ALifeTraderAbstract::specific_character` resolves it: by the
  /// specific character its spawn names, or else by its profile. Answers the profile, where one is read.
  fn verify_spawned_trader(
    partners: &DialogPartners,
    object: &AlifeObject,
    path: &Path,
    result: &mut GamedataDialogsVerificationResult,
  ) -> Option<String> {
    let trader: &AlifeObjectTraderAbstract = object.inherited.get_trader()?;
    let npc: String = format!("NPC '{}' of section [{}]", object.header.name, object.header.section);
    let character: &str = trader.specific_character.as_str();

    result.checked_references_count += 1;

    // A named character is used as it is, and its profile is never read.
    if !character.is_empty() {
      if !partners.get_characters().contains_key(character) {
        result.missing_references_count += 1;
        result.fail(
          GamedataVerificationRule::DialogSpawnUnresolved,
          path,
          format!("{npc} is spawned as character '{character}', which no character file declares"),
        );
      }

      return None;
    }

    let id: &str = trader.character_profile.as_str();
    let message: Option<String> = match partners.find_profile(id) {
      None => Some(format!(
        "{npc} is spawned with profile '{id}', which no profile file declares"
      )),
      Some(profile) if !partners.is_profile_resolvable(profile) => Some(format!(
        "{npc} is spawned with a profile no character resolves: {}",
        Self::describe_unresolved(profile)
      )),
      Some(_) => None,
    };

    if let Some(message) = message {
      result.missing_references_count += 1;
      result.fail(GamedataVerificationRule::DialogSpawnUnresolved, path, message);
    }

    Some(id.to_owned())
  }

  /// Every dialog the `meet` and `wounded` sections of an object's custom data name is declared.
  ///
  /// Custom data that does not parse names nothing here; the particles check reports it.
  fn verify_spawned_logic(
    &self,
    declared: &HashMap<&str, &Dialog>,
    object: &AlifeObject,
    path: &Path,
    result: &mut GamedataDialogsVerificationResult,
  ) {
    let Some(custom_data) = object.inherited.get_custom_data() else {
      return;
    };

    if custom_data.trim().is_empty() {
      return;
    }

    let Ok(ltx) = Ltx::read_from_str(custom_data) else {
      return;
    };
    let owner: String = format!("NPC '{}'", object.header.name);

    for reference in DialogLogicReference::list_in(&ltx) {
      self.verify_logic_reference(declared, &reference, path, Some(&owner), result);
    }
  }
}
