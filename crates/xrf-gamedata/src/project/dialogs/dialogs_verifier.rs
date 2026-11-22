use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use xrf_dialog::{
  Dialog, DialogPartners, DialogPhrase, DialogProject, DialogProjectLayout, DialogProjectMode, DialogTextIndex,
};
use xrf_error::XrfResult;
use xrf_ltx::LtxResolution;
use xrf_vfs::{XrayLogicalPath, XrayMountMode, XrayRoots, XraySearch};

use crate::project::dialogs::dialog_logic_reference::DialogLogicReference;
use crate::project::dialogs::dialog_phrase_graph::DialogPhraseGraph;
use crate::project::dialogs::dialog_system_config::DialogSystemConfig;
use crate::project::dialogs::verify_dialogs_result::GamedataDialogsVerificationResult;
use crate::project::gamedata_check::GamedataCheck;
use crate::project::gamedata_finding_factory::GamedataFindingFactory;
use crate::project::gamedata_project::GamedataProject;
use crate::project::gamedata_project_verify_options::GamedataProjectVerifyOptions;
use crate::project::gamedata_verification_rule::GamedataVerificationRule;

mod dialogs_verifier_files;

/// The dialogs check: every dialog opens and walks as the engine loads it, and every character, profile, info portion
/// and logic section offering one names what the tree declares.
///
/// What the engine asserts on fails the check; what it tolerates is reported, and fails only under strict.
pub(crate) struct DialogsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for DialogsVerifier<'a> {
  type Result = GamedataDialogsVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataDialogsVerificationResult> {
    self.options.job.check_cancelled()?;

    let started_at: Instant = Instant::now();
    let search: &XraySearch = self.project.search();
    let mut result: GamedataDialogsVerificationResult = GamedataDialogsVerificationResult::default();

    if DialogProject::list_dialog_assets(&search.clone().with_prefix(DialogProjectMode::DIALOGS_PREFIX)?).is_empty() {
      result.duration = started_at.elapsed();

      return Ok(result);
    }

    // The roots only describe the project; reading goes through the search every check shares.
    let roots: XrayRoots = XrayRoots::one(self.project.root().to_path_buf(), XrayMountMode::Auto);
    let layout: DialogProjectLayout = DialogProjectLayout::new(DialogProjectMode::Gamedata);
    let system_ltx: Arc<LtxResolution> = self.project.ltx_project.system_ltx()?;
    let system: DialogSystemConfig = DialogSystemConfig::read(&system_ltx.ltx);
    let project: DialogProject = match &system.listed {
      Some(listed) => DialogProject::from_listed(search.clone(), &roots, &layout, listed),
      None => DialogProject::from_search(search.clone(), &roots, &layout)?,
    };
    let partners: DialogPartners = match &system.listed {
      Some(listed) => DialogPartners::from_listed(search, DialogProjectMode::DIALOGS_PREFIX, listed),
      None => DialogPartners::from_search(search, DialogProjectMode::DIALOGS_PREFIX)?,
    };
    let declared: HashMap<&str, &Dialog> = Self::index_declared(&project);

    result.has_dialogs = true;

    self.report_dialog_files(&project, system.listed.as_ref(), &mut result)?;
    self.report_duplicate_dialogs(&project, &declared, &mut result);
    self.report_partner_files(&partners, system.listed.as_ref(), &mut result)?;
    self.options.job.check_cancelled()?;
    self.verify_dialogs(&project, &declared, &mut result);
    self.options.job.check_cancelled()?;
    self.verify_offers(&partners, &declared, &mut result);
    self.verify_profiles(&partners, &system, &mut result);
    self.verify_characters(&partners, &mut result);
    self.options.job.check_cancelled()?;
    self.verify_logic(&declared, &mut result);
    self.options.job.check_cancelled()?;

    result
      .findings
      .sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);
    result.duration = started_at.elapsed();

    Ok(result)
  }
}

impl DialogsVerifier<'_> {
  /// Every dialog id by its first declaration in file order, which is the one the engine's id index keeps.
  fn index_declared(project: &DialogProject) -> HashMap<&str, &Dialog> {
    let mut declared: HashMap<&str, &Dialog> = HashMap::new();

    for file in project.get_files() {
      for dialog in file.get_file().get_dialogs() {
        declared.entry(dialog.get_id()).or_insert(dialog);
      }
    }

    declared
  }

  /// Whether a dialog is the first declaration of its id, the one the engine reads.
  fn is_declaration_read(declared: &HashMap<&str, &Dialog>, dialog: &Dialog) -> bool {
    declared
      .get(dialog.get_id())
      .is_some_and(|first| std::ptr::eq(*first, dialog))
  }

  /// Each dialog's phrase graph as the engine builds it, and the lines of the phrases it reaches.
  fn verify_dialogs(
    &self,
    project: &DialogProject,
    declared: &HashMap<&str, &Dialog>,
    result: &mut GamedataDialogsVerificationResult,
  ) {
    for file in project.get_files() {
      for dialog in file.get_file().get_dialogs() {
        // A later declaration of an id is one the engine never reads.
        if !Self::is_declaration_read(declared, dialog) {
          continue;
        }

        let path: PathBuf = self.report_path(file.get_logical_path());
        let failures_before: usize = result.count_failures();

        result.checked_dialogs_count += 1;
        Self::verify_graph(dialog, project.get_text(), &path, result);

        if result.count_failures() > failures_before {
          result.invalid_dialogs_count += 1;
        }
      }
    }
  }

  fn verify_graph(
    dialog: &Dialog,
    text: &DialogTextIndex,
    path: &Path,
    result: &mut GamedataDialogsVerificationResult,
  ) {
    let id: &str = dialog.get_id();
    let graph: DialogPhraseGraph<'_> = DialogPhraseGraph::new(dialog);

    if graph.is_scripted() {
      if !graph.has_init_func() {
        result.fail(
          GamedataVerificationRule::DialogPhraseEntry,
          path,
          format!("Dialog '{id}' has no phrase list and no init_func to build one"),
        );
      }

      return;
    }

    if graph.find(Dialog::ENTRY_PHRASE_ID).is_none() {
      result.fail(
        GamedataVerificationRule::DialogPhraseEntry,
        path,
        format!(
          "Dialog '{id}' lists no phrase '{}', which the engine opens it with",
          Dialog::ENTRY_PHRASE_ID
        ),
      );

      return;
    }

    let reached: Vec<&DialogPhrase> = graph.list_reached();

    for phrase in &reached {
      for next in phrase.list_next() {
        if graph.find(next).is_none() {
          result.fail(
            GamedataVerificationRule::DialogPhraseNext,
            path,
            format!(
              "Dialog '{id}' phrase '{}' leads to phrase '{next}', which the dialog does not list",
              phrase.get_id()
            ),
          );
        }
      }

      Self::verify_text(id, phrase, text, path, result);
    }

    let unreached: Vec<&str> = graph.list_unreached_ids(&reached);

    if !unreached.is_empty() {
      result.tolerate(
        GamedataVerificationRule::DialogPhraseUnreachable,
        path,
        format!(
          "Dialog '{id}' phrases {} are not reached from phrase '{}', so the engine never loads them",
          Self::quote_all(&unreached),
          Dialog::ENTRY_PHRASE_ID
        ),
      );
    }
  }

  /// A phrase line has text in every language the game ships; the engine shows the key where it has none.
  fn verify_text(
    id: &str,
    phrase: &DialogPhrase,
    text: &DialogTextIndex,
    path: &Path,
    result: &mut GamedataDialogsVerificationResult,
  ) {
    let Some(key) = phrase.get_text() else {
      return;
    };

    if text.get_languages().is_empty() {
      return;
    }

    if !text.contains_key(key) {
      result.tolerate(
        GamedataVerificationRule::DialogTextMissing,
        path,
        format!(
          "Dialog '{id}' phrase '{}' text '{key}' is defined in no language",
          phrase.get_id()
        ),
      );

      return;
    }

    let missing: Vec<&str> = text
      .get_languages()
      .iter()
      .map(String::as_str)
      .filter(|language| text.resolve(key, language).is_none())
      .collect();

    if !missing.is_empty() {
      result.tolerate(
        GamedataVerificationRule::DialogTextMissing,
        path,
        format!(
          "Dialog '{id}' phrase '{}' text '{key}' has no line in {}",
          phrase.get_id(),
          missing.join(", ")
        ),
      );
    }
  }

  /// Every start and actor dialog of a character, and every dialog of an info portion, is declared.
  fn verify_offers(
    &self,
    partners: &DialogPartners,
    declared: &HashMap<&str, &Dialog>,
    result: &mut GamedataDialogsVerificationResult,
  ) {
    for character in partners.get_characters().values() {
      let path: PathBuf = self.report_path(character.get_logical_path());
      let id: &str = character.get_id();

      if let Some(dialog) = character.get_start_dialog() {
        Self::verify_offer(declared, &path, dialog, result, || {
          format!("Character '{id}' offers start dialog '{dialog}', which no dialog file declares")
        });
      }

      for dialog in character.get_actor_dialogs() {
        Self::verify_offer(declared, &path, dialog, result, || {
          format!("Character '{id}' offers actor dialog '{dialog}', which no dialog file declares")
        });
      }
    }

    for portion in partners.get_info_portions().values() {
      let path: PathBuf = self.report_path(portion.get_logical_path());

      for dialog in portion.get_dialogs() {
        Self::verify_offer(declared, &path, dialog, result, || {
          format!(
            "Info portion '{}' opens dialog '{dialog}', which no dialog file declares",
            portion.get_id()
          )
        });
      }
    }
  }

  /// One offered dialog is declared, or a finding the offer describes.
  fn verify_offer(
    declared: &HashMap<&str, &Dialog>,
    path: &Path,
    dialog: &str,
    result: &mut GamedataDialogsVerificationResult,
    describe: impl FnOnce() -> String,
  ) {
    result.checked_references_count += 1;

    if !declared.contains_key(dialog) {
      result.missing_references_count += 1;
      result.fail(GamedataVerificationRule::DialogOfferMissing, path, describe());
    }
  }

  /// Every profile resolves to a character an NPC can be spawned as, and every profile a section names is declared.
  ///
  /// Both stop the engine only when an NPC is spawned with the profile, and neither a profile nor a section says whether
  /// one is: vanilla ships fifty `test_npc_*` profiles and `stalker_promo_*` sections nothing spawns, and monster sections
  /// whose `character_profile` the engine never reads. So both are reported, failing only under strict.
  fn verify_profiles(
    &self,
    partners: &DialogPartners,
    system: &DialogSystemConfig,
    result: &mut GamedataDialogsVerificationResult,
  ) {
    // todo: Fail these where the spawn files place an NPC of the section or profile, which is when the engine asserts.
    for profile in partners.get_profiles().values() {
      result.checked_profiles_count += 1;

      if partners.is_profile_resolvable(profile) {
        continue;
      }

      let id: &str = profile.get_id();
      let problem: String = match (profile.get_specific_character(), profile.get_class()) {
        (Some(character), _) => {
          format!("Profile '{id}' pins character '{character}', which no character file declares")
        }
        (None, Some(class)) => {
          format!("Profile '{id}' picks by class '{class}', which no team_default character without no_random has")
        }
        (None, None) => format!("Profile '{id}' picks any character, and none without no_random is team_default"),
      };
      let path: PathBuf = self.report_path(profile.get_logical_path());

      let message: String = match system.profile_sections.get(id) {
        Some(section) => format!("{problem}, and section [{section}] names it as its character_profile"),
        None => problem,
      };

      result.unresolved_profiles_count += 1;
      result.tolerate(GamedataVerificationRule::DialogProfileUnresolved, &path, message);
    }

    let Ok(system_path) = self.project.ltx_project.system_ltx_report_path() else {
      return;
    };

    for (profile, section) in &system.profile_sections {
      result.checked_references_count += 1;

      if partners.find_profile(profile).is_none() {
        result.missing_references_count += 1;
        result.tolerate(
          GamedataVerificationRule::DialogProfileUnresolved,
          &system_path,
          format!("Section [{section}] character_profile names profile '{profile}', which no profile file declares"),
        );
      }
    }
  }

  /// No character carries both flags `CSpecificCharacter::load_shared` refuses together.
  fn verify_characters(&self, partners: &DialogPartners, result: &mut GamedataDialogsVerificationResult) {
    for character in partners.get_characters().values() {
      if !character.is_random() && character.is_team_default() {
        result.fail(
          GamedataVerificationRule::DialogCharacterFlags,
          &self.report_path(character.get_logical_path()),
          format!(
            "Character '{}' is marked both no_random and team_default, which the engine asserts on loading it",
            character.get_id()
          ),
        );
      }
    }
  }

  /// Every dialog a `meet` or `wounded` logic section names is declared, each named once however many configs include
  /// its section.
  fn verify_logic(&self, declared: &HashMap<&str, &Dialog>, result: &mut GamedataDialogsVerificationResult) {
    let outcomes: Vec<(PathBuf, XrfResult<Vec<DialogLogicReference>>)> = self
      .project
      .visit_configs(&self.options.job, |ltx, _| DialogLogicReference::list_in(ltx));
    let mut seen: HashSet<DialogLogicReference> = HashSet::new();

    // A config that does not resolve is reported by the LTX check, not this one.
    for (path, outcome) in outcomes {
      let Ok(references) = outcome else {
        continue;
      };

      for reference in references {
        if !seen.insert(reference.clone()) {
          continue;
        }

        result.checked_references_count += 1;

        if declared.contains_key(reference.dialog.as_str()) {
          continue;
        }

        result.missing_references_count += 1;

        let message: String = format!(
          "[{}] {} names dialog '{}', which no dialog file declares",
          reference.section, reference.field, reference.dialog
        );

        // Each rule keeps one severity, so a report reader can tell a failing finding by its rule alone.
        if reference.is_start_dialog {
          result.fail(GamedataVerificationRule::DialogLogicMissing, &path, message);
        } else {
          result.tolerate(GamedataVerificationRule::DialogHealMissing, &path, message);
        }
      }
    }
  }

  /// The path a finding names for a file of the tree: the physical one where it is loose, so a reader can open it.
  fn report_path(&self, logical_path: &str) -> PathBuf {
    XrayLogicalPath::new(logical_path)
      .map(|path| self.project.ltx_project.path_of(&path))
      .unwrap_or_else(|_| PathBuf::from(logical_path))
  }

  fn quote_all(ids: &[&str]) -> String {
    ids
      .iter()
      .map(|id| format!("'{id}'"))
      .collect::<Vec<String>>()
      .join(", ")
  }
}
