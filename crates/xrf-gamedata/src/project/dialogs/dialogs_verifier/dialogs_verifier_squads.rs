use std::collections::HashSet;

use xrf_dialog::DialogPartners;
use xrf_ltx::Ltx;

use crate::project::dialogs::dialogs_verifier::DialogsVerifier;
use crate::project::dialogs::squads::dialog_squad_member::DialogSquadMember;
use crate::project::dialogs::squads::dialog_squad_spawns::DialogSquadSpawns;
use crate::project::dialogs::verify_dialogs_result::GamedataDialogsVerificationResult;
use crate::project::gamedata_verification_rule::GamedataVerificationRule;

/// Who the squads the configs spawn place: the profile each of their NPCs is spawned with.
impl DialogsVerifier<'_> {
  /// Every squad the configs spawn is declared, every NPC section a squad lists is declared, and every profile a
  /// spawned squad places an NPC with resolves to a character.
  ///
  /// `alife():create` asserts on a section no config declares, and `CSE_ALifeTraderAbstract::specific_character`
  /// resolves the NPC's character as the squad creates it, asserting on a profile no file declares or no character
  /// resolves, as it does for an NPC the spawn places. An NPC section only squads nothing spawns list is a warning: the
  /// engine meets it only if a script creates the squad. Answers the profiles, whose findings these are.
  pub(super) fn verify_squads(
    &self,
    partners: &DialogPartners,
    system: &Ltx,
    result: &mut GamedataDialogsVerificationResult,
  ) -> HashSet<String> {
    let squads: DialogSquadSpawns = DialogSquadSpawns::read(self.project, system, &self.options.job);
    let Ok(path) = self.project.ltx_project.system_ltx_report_path() else {
      return HashSet::new();
    };

    for (squad, named_in) in &squads.missing_squads {
      result.checked_references_count += 1;
      result.missing_references_count += 1;
      result.fail(
        GamedataVerificationRule::DialogSquadMissing,
        named_in,
        format!("Squad [{squad}] is spawned here, but no section declares it"),
      );
    }

    for member in &squads.missing_members {
      result.checked_references_count += 1;
      result.missing_references_count += 1;
      result.fail(
        GamedataVerificationRule::DialogSquadMissing,
        &path,
        Self::describe_missing_member(member),
      );
    }

    for member in &squads.unspawned_missing_members {
      result.checked_references_count += 1;
      result.missing_references_count += 1;
      result.tolerate(
        GamedataVerificationRule::DialogSquadMissing,
        &path,
        format!("{}; no config spawns the squad", Self::describe_missing_member(member)),
      );
    }

    for (id, member) in &squads.profiles {
      let npc: String = format!("Squad [{}] places NPC section [{}]", member.squad, member.section);
      let message: Option<String> = match partners.find_profile(id) {
        None => Some(format!("{npc} with profile '{id}', which no profile file declares")),
        Some(profile) if !partners.is_profile_resolvable(profile) => Some(format!(
          "{npc} with a profile no character resolves: {}",
          Self::describe_unresolved(profile)
        )),
        Some(_) => None,
      };

      result.checked_references_count += 1;

      if let Some(message) = message {
        result.missing_references_count += 1;
        result.fail(GamedataVerificationRule::DialogSpawnUnresolved, &path, message);
      }
    }

    squads.profiles.into_keys().collect()
  }

  /// What a squad listing an NPC section no one declares is told.
  fn describe_missing_member(member: &DialogSquadMember) -> String {
    format!(
      "Squad [{}] lists NPC section [{}], which no section declares",
      member.squad, member.section
    )
  }
}
