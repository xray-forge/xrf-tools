use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_collections::indexmap::{IndexMap, IndexSet};
use xrf_dialog::{
  DialogElementKind, DialogListedFiles, DialogPartners, DialogProject, DialogProjectLayout, DialogProjectMode,
  DialogReferenceDescriptor,
};
use xrf_error::XrfResult;
use xrf_job::JobHandle;
use xrf_ltx::{Ltx, LtxResolution};
use xrf_spawn::SpawnFile;
use xrf_vfs::{XrayLogicalPath, XrayMountMode, XrayRoots, XraySearch};

use crate::project::gamedata_project::GamedataProject;
use crate::project::info_portions::info_portion_mention::InfoPortionMention;
use crate::project::info_portions::info_portion_place::InfoPortionPlace;
use crate::project::info_portions::info_portion_role::InfoPortionRole;
use crate::project::info_portions::info_portion_script_names::InfoPortionScriptNames;
use crate::project::info_portions::info_portion_uses::InfoPortionUses;
use crate::project::scripts::gamedata_parsed_script::GamedataParsedScript;
use crate::project::scripts::scripts_verifier::ScriptsVerifier;

/// Every info portion a tree names, and how: where the info portion files declare it, where condlists, dialogs and
/// spawned documents give, take and read it, and whether a script or a condlist function may give it out of sight.
///
/// The engine keeps an info portion as a flag the actor holds, declared or not (`CInfoPortion::load_shared` returns
/// quietly for an undeclared id in Call of Pripyat), so the uses only hold together by their spelling.
#[derive(Default)]
pub(crate) struct InfoPortionTable {
  uses: IndexMap<String, InfoPortionUses>,
  /// Names condlist functions receive, which mark the portions other sources name once all are read.
  passed: HashSet<String>,
  /// Identifier-shaped names scripts pass to calls or keep as data, in first-met order.
  script_names: IndexSet<String>,
  /// Runtime scripts the tree holds, read or not.
  scripts_count: u32,
  /// Runtime scripts that did not read or parse, whose names are not known.
  unread_scripts_count: u32,
  /// Configs that did not resolve, whose condlists are not known.
  unread_configs_count: u32,
}

impl InfoPortionTable {
  /// The info portions Call of Pripyat's engine gives the actor itself: the PDA opening and closing (`xray-16`
  /// `src/xrGame/ui/UIPdaWnd.cpp`), the talk window showing and hiding (`UITalkDialogWnd.cpp`), and the PDA map's local
  /// view (`UIMapWnd.cpp`). Shadow of Chernobyl's `ui_inventory`, `ui_trade`, `ui_car_body` and `ui_talk`, which vanilla
  /// still declares, are no longer among them.
  const ENGINE_GIVEN: [&'static str; 5] = [
    "ui_pda",
    "ui_pda_hide",
    "ui_talk_show",
    "ui_talk_hide",
    "ui_pda_map_local",
  ];
  /// The debug scenes vanilla's developer menu jumps into, each starting with an info portion it gives.
  const SCENES_PATH: &'static str = "configs\\scripts\\scenes.ltx";
  /// What separates a debug scene's fields: its info portion, walk and look paths, and items.
  const SCENE_FIELD_SEPARATOR: char = '|';

  /// Read every use a tree writes: its info portion and dialog files as `system.ltx` lists them, its configs, its spawn
  /// files with their objects' custom data, and its scripts.
  ///
  /// # Errors
  ///
  /// Returns an error when the dialogs prefix is not a valid logical path.
  pub(crate) fn read(project: &GamedataProject, job: &JobHandle) -> XrfResult<Self> {
    let mut table: Self = Self::default();
    let system: Option<Arc<LtxResolution>> = project.ltx_project.system_ltx().ok();
    let listed: Option<DialogListedFiles> = system
      .as_ref()
      .and_then(|system| DialogListedFiles::from_system_ltx(&system.ltx));

    table.add_dialogs(project, listed.as_ref())?;
    table.add_configs(project, job);
    table.add_debug_scenes(project);
    table.add_spawns(project, job);
    table.add_scripts(project, job);
    table.mark_passed();
    table.mark_given_by_engine();

    Ok(table)
  }

  /// Each info portion the tree names, with its uses, in the order it is first met.
  pub(crate) fn get_uses(&self) -> &IndexMap<String, InfoPortionUses> {
    &self.uses
  }

  /// The names a misspelling may have been meant as: those something gives, and those scripts name, in a fixed order.
  pub(crate) fn list_known_names(&self) -> Vec<&str> {
    let mut names: Vec<&str> = self
      .uses
      .iter()
      .filter(|(_, uses)| uses.given.is_some())
      .map(|(name, _)| name.as_str())
      .chain(self.script_names.iter().map(String::as_str))
      .collect();

    names.sort_unstable();
    names.dedup();
    names
  }

  pub(crate) fn get_scripts_count(&self) -> u32 {
    self.scripts_count
  }

  pub(crate) fn get_unread_scripts_count(&self) -> u32 {
    self.unread_scripts_count
  }

  pub(crate) fn get_unread_configs_count(&self) -> u32 {
    self.unread_configs_count
  }

  /// The info portions the info portion files declare and the dialogs give, take and read, read from the files
  /// `system.ltx` lists, or by name where it lists none, and those the dialog files it leaves out would give. A
  /// condition the engine never reads, on an entry phrase no phrase leads back to, reads nothing.
  fn add_dialogs(&mut self, project: &GamedataProject, listed: Option<&DialogListedFiles>) -> XrfResult {
    let search: XraySearch = project.search().clone();
    let roots: XrayRoots = XrayRoots::one(project.root().to_path_buf(), XrayMountMode::Auto);
    let layout: DialogProjectLayout = DialogProjectLayout {
      is_text_skipped: true,
      ..DialogProjectLayout::new(DialogProjectMode::Gamedata)
    };
    // A tree holding no dialog files, such as a mod touching only configs, has none to read.
    let named: Option<DialogProject> = DialogProject::from_search(search.clone(), &roots, &layout).ok();
    let listed_dialogs: Option<DialogProject> =
      listed.map(|listed| DialogProject::from_listed(search, &roots, &layout, listed));
    let dialogs: Option<&DialogProject> = listed_dialogs.as_ref().or(named.as_ref());
    // The partners the lists name, or those named as partner files, as either project would read them.
    let partners: &DialogPartners = project.get_dialog_partners()?;

    for info_portion in partners.get_info_portions().values() {
      let id: &str = info_portion.get_id();
      let path: PathBuf = Self::resolve_report_path(project, info_portion.get_logical_path());

      self.record_use(
        id,
        InfoPortionRole::Declared,
        InfoPortionPlace {
          path: path.clone(),
          context: format!("info_portion '{id}'"),
        },
      );

      // Holding it opens its dialogs, and giving it takes its `<disable>` list away (`xray-16` `InfoPortion.cpp`).
      if !info_portion.get_dialogs().is_empty() || !info_portion.get_disables().is_empty() {
        self.uses.entry(id.to_owned()).or_default().is_effective = true;
      }

      for disabled in info_portion.get_disables() {
        self.record_use(
          disabled,
          InfoPortionRole::Taken,
          InfoPortionPlace {
            path: path.clone(),
            context: format!("info_portion '{id}' <disable>"),
          },
        );
      }
    }

    if let Some(dialogs) = dialogs {
      for reference in Self::list_info_references(dialogs) {
        let role: InfoPortionRole = match reference.element.kind {
          DialogElementKind::GiveInfo => InfoPortionRole::Given,
          DialogElementKind::DisableInfo => InfoPortionRole::Taken,
          _ => InfoPortionRole::Read,
        };

        self.add_dialog_reference(project, &reference, role);
      }
    }

    // What a dialog file the lists leave out gives tells a reader why a portion it reads is never given.
    if let Some(listed) = listed
      && let Some(named) = &named
    {
      for reference in Self::list_info_references(named) {
        if reference.element.kind == DialogElementKind::GiveInfo
          && !listed.is_dialog_file(DialogProjectMode::DIALOGS_PREFIX, &reference.logical_path)
        {
          self.add_dialog_reference(project, &reference, InfoPortionRole::GivenUnlisted);
        }
      }
    }

    Ok(())
  }

  /// Every info portion element of a project's dialogs the engine reads.
  fn list_info_references(dialogs: &DialogProject) -> Vec<DialogReferenceDescriptor> {
    dialogs
      .list_references(|element| element.get_kind().is_info_portion())
      .into_iter()
      .filter(|reference| !reference.is_ignored)
      .collect()
  }

  fn add_dialog_reference(
    &mut self,
    project: &GamedataProject,
    reference: &DialogReferenceDescriptor,
    role: InfoPortionRole,
  ) {
    let mut context: String = match &reference.phrase_id {
      Some(phrase) => format!("dialog '{}' phrase {phrase}", reference.dialog_id),
      None => format!("dialog '{}'", reference.dialog_id),
    };

    // A finding about another file names the one this dialog lies in, by the path the engine would open it at.
    if role == InfoPortionRole::GivenUnlisted {
      context.push_str(&format!(" of {}", reference.logical_path));
    }

    self.record_use(
      &reference.element.value,
      role,
      InfoPortionPlace {
        path: Self::resolve_report_path(project, &reference.logical_path),
        context,
      },
    );
  }

  /// The info portions every config's condlists read, give, take and pass. A config that does not resolve is reported
  /// by the LTX check and counted here.
  fn add_configs(&mut self, project: &GamedataProject, job: &JobHandle) {
    for (path, outcome) in project.visit_configs(job, |ltx, _| InfoPortionMention::list_in_ltx(ltx, None)) {
      let Ok(mentions) = outcome else {
        self.unread_configs_count += 1;
        continue;
      };

      for mention in mentions {
        self.add_mention(mention, &path);
      }
    }
  }

  /// The info portions the debug scenes of `scripts\scenes.ltx` start with: each level's section maps a scene to
  /// `info | walk | look | items`, and vanilla's developer menu (`ui_scenes.script`) gives the first field to jump into
  /// a quest, so a quest reading it is the author's entry point, not a misspelling.
  fn add_debug_scenes(&mut self, project: &GamedataProject) {
    let Ok(path) = XrayLogicalPath::new(Self::SCENES_PATH) else {
      return;
    };
    // A tree without the file has no scenes; one that does not resolve is reported by the LTX check.
    let Ok(scenes) = project.ltx_project.read_resolution(&path) else {
      return;
    };
    let reported: PathBuf = project.ltx_project.path_of(&path);

    for (level, section) in &scenes.ltx {
      for (scene, value) in section {
        let Some(info_portion) = value
          .split(Self::SCENE_FIELD_SEPARATOR)
          .next()
          .map(str::trim)
          .filter(|info_portion| !info_portion.is_empty())
        else {
          continue;
        };

        self.record_use(
          info_portion,
          InfoPortionRole::GivenByDebugScene,
          InfoPortionPlace {
            path: reported.clone(),
            context: format!("debug scene '{scene}' of [{level}] in scripts\\scenes.ltx"),
          },
        );
      }
    }
  }

  /// The info portions the spawn files' documents give, and those the condlists of their objects' custom data read,
  /// give, take and pass. A spawn file that does not read is reported by the spawns check.
  fn add_spawns(&mut self, project: &GamedataProject, job: &JobHandle) {
    for spawn_path in project.spawn_paths() {
      if job.is_cancelled() {
        return;
      }

      let Ok(spawn) = project.read_parsed(&spawn_path, SpawnFile::read_from_chunk) else {
        continue;
      };
      let path: PathBuf = Self::resolve_report_path(project, &spawn_path);

      for object in &spawn.alife_spawn.objects {
        let owner: String = format!("object '{}'", object.header.name);

        if let Some(info_portion) = object.inherited.get_info_portion() {
          self.add_mention(
            InfoPortionMention::new(info_portion, InfoPortionRole::Given, &owner),
            &path,
          );
        }

        // Custom data that does not parse names nothing here; the particles usage check reports it.
        let Some(Ok(ltx)) = object
          .inherited
          .get_custom_data()
          .filter(|custom_data| !custom_data.trim().is_empty())
          .map(|custom_data| Ltx::read_from_str(custom_data))
        else {
          continue;
        };

        for mention in InfoPortionMention::list_in_ltx(&ltx, Some(&owner)) {
          self.add_mention(mention, &path);
        }
      }
    }
  }

  /// What scripts do with the names other sources name (`InfoPortionScriptNames`). A script that does not read or
  /// parse is reported by the scripts check and counted here.
  fn add_scripts(&mut self, project: &GamedataProject, job: &JobHandle) {
    let parsed: Vec<GamedataParsedScript> = ScriptsVerifier::parse_runtime_scripts(project, job);
    let mut names: InfoPortionScriptNames<'_> = InfoPortionScriptNames::read(project, &parsed);

    self.scripts_count = u32::try_from(parsed.len()).unwrap_or(u32::MAX);
    self.unread_scripts_count = u32::try_from(names.unread_count).unwrap_or(u32::MAX);

    // A string no other source names is not an info portion this table follows.
    for (name, uses) in &mut self.uses {
      if let Some(place) = names.given.remove(name.as_str()) {
        uses.given.get_or_insert(place);
      }

      uses.is_script_read = names.read.contains(name.as_str());
      uses.is_scripted = names.is_unknown(name);
    }

    self.script_names = names.list_identifiers().map(str::to_owned).collect();
  }

  /// Marks the portions a condlist function receives, once every source has named what it names.
  fn mark_passed(&mut self) {
    for (name, uses) in &mut self.uses {
      uses.is_passed = self.passed.contains(name);
    }
  }

  /// Marks the portions the engine gives itself, as Call of Pripyat's windows send them to the actor
  /// (`InventoryUtilities::SendInfoToActor`).
  fn mark_given_by_engine(&mut self) {
    for name in Self::ENGINE_GIVEN {
      if let Some(uses) = self.uses.get_mut(name) {
        uses.is_given_by_engine = true;
      }
    }
  }

  fn add_mention(&mut self, mention: InfoPortionMention, path: &Path) {
    self.record_use(
      &mention.name,
      mention.role,
      InfoPortionPlace {
        path: path.to_path_buf(),
        context: mention.context,
      },
    );
  }

  /// Records one use, keeping the first place of each kind. A name a condlist function receives is only kept aside: a
  /// squad, a smart terrain or a number is passed as often, so it names an info portion only where another use does.
  fn record_use(&mut self, name: &str, role: InfoPortionRole, place: InfoPortionPlace) {
    if role == InfoPortionRole::Passed {
      self.passed.insert(name.to_owned());

      return;
    }

    let uses: &mut InfoPortionUses = self.uses.entry(name.to_owned()).or_default();

    if let Some(first) = Self::find_first_place(uses, role) {
      first.get_or_insert(place);
    }
  }

  /// Where the first use of a role is kept, or `None` for a role no place is kept for.
  fn find_first_place(uses: &mut InfoPortionUses, role: InfoPortionRole) -> Option<&mut Option<InfoPortionPlace>> {
    match role {
      InfoPortionRole::Declared => Some(&mut uses.declared),
      InfoPortionRole::Given => Some(&mut uses.given),
      InfoPortionRole::GivenUnlisted => Some(&mut uses.given_unlisted),
      InfoPortionRole::GivenByDebugScene => Some(&mut uses.given_by_debug_scene),
      InfoPortionRole::Taken => Some(&mut uses.taken),
      InfoPortionRole::Read => Some(&mut uses.read),
      InfoPortionRole::Passed => None,
    }
  }

  /// The path a finding names: the file on disk where there is one, else the logical path.
  pub(crate) fn resolve_report_path(project: &GamedataProject, logical_path: &str) -> PathBuf {
    XrayLogicalPath::new(logical_path)
      .map(|path| project.ltx_project.path_of(&path))
      .unwrap_or_else(|_| PathBuf::from(logical_path))
  }
}
