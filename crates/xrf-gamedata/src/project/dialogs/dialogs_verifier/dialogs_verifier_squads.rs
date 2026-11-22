use std::collections::HashSet;

use xrf_dialog::DialogPartners;
use xrf_ltx::Ltx;

use crate::project::dialogs::dialogs_verifier::DialogsVerifier;
use crate::project::dialogs::squads::dialog_squad_spawns::DialogSquadSpawns;
use crate::project::dialogs::verify_dialogs_result::GamedataDialogsVerificationResult;
use crate::project::gamedata_verification_rule::GamedataVerificationRule;

/// Who the squads the configs spawn place: the profile each of their NPCs is spawned with.
impl DialogsVerifier<'_> {
  /// Every profile a squad the configs spawn places an NPC with resolves to a character.
  ///
  /// `CSE_ALifeTraderAbstract::specific_character` resolves the NPC's character as the squad creates it, and asserts on a
  /// profile no file declares or no character resolves, as it does for an NPC the spawn places. Answers the profiles,
  /// whose findings these are.
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
}
