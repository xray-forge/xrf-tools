use indexmap::IndexMap;
use xrf_dialog::DialogListedFiles;
use xrf_ltx::Ltx;

/// What `system.ltx`, with its includes, tells the dialogs check: which dialog and partner files the engine loads, and
/// which profiles its sections name for the NPCs spawned from them.
pub(crate) struct DialogSystemConfig {
  /// The files the engine loads, or `None` for a tree whose `system.ltx` lists none, such as a mod carrying only part
  /// of a game. A list it leaves out is empty, as the engine reads it.
  pub(crate) listed: Option<DialogListedFiles>,
  /// Each profile a section names as its `character_profile`, by the first section naming it.
  pub(crate) profile_sections: IndexMap<String, String>,
}

impl DialogSystemConfig {
  /// The key `CSE_ALifeTraderAbstract` reads an NPC's profile from.
  const CHARACTER_PROFILE_KEY: &'static str = "character_profile";

  pub(crate) fn read(ltx: &Ltx) -> Self {
    let mut profile_sections: IndexMap<String, String> = IndexMap::new();

    for (name, section) in ltx {
      if let Some(profile) = section
        .get(Self::CHARACTER_PROFILE_KEY)
        .map(str::trim)
        .filter(|profile| !profile.is_empty())
      {
        profile_sections
          .entry(profile.to_owned())
          .or_insert_with(|| name.to_owned());
      }
    }

    Self {
      listed: DialogListedFiles::from_system_ltx(ltx),
      profile_sections,
    }
  }
}
