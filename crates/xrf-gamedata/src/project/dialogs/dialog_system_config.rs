use indexmap::IndexMap;
use xrf_dialog::DialogPartnerFiles;
use xrf_ltx::Ltx;

/// What `system.ltx`, with its includes, tells the dialogs check: which partner files the engine loads, and which
/// profiles its sections name for the NPCs spawned from them.
pub(crate) struct DialogSystemConfig {
  /// The partner files the engine loads, or `None` for a tree whose `system.ltx` lists none, such as a mod carrying
  /// only part of a game.
  pub(crate) partner_files: Option<DialogPartnerFiles>,
  /// Each profile a section names as its `character_profile`, by the first section naming it.
  pub(crate) profile_sections: IndexMap<String, String>,
}

impl DialogSystemConfig {
  const PROFILES_SECTION: &'static str = "profiles";
  const INFO_PORTIONS_SECTION: &'static str = "info_portions";
  const FILES_KEY: &'static str = "files";
  const CHARACTERS_FILES_KEY: &'static str = "specific_characters_files";
  /// The key `CSE_ALifeTraderAbstract` reads an NPC's profile from.
  const CHARACTER_PROFILE_KEY: &'static str = "character_profile";

  pub(crate) fn read(ltx: &Ltx) -> Self {
    let list = |section: &str, key: &str| -> Option<Vec<String>> {
      ltx
        .section(section)
        .and_then(|section| section.get(key))
        .map(DialogPartnerFiles::split_list)
    };

    let characters: Option<Vec<String>> = list(Self::PROFILES_SECTION, Self::CHARACTERS_FILES_KEY);
    let profiles: Option<Vec<String>> = list(Self::PROFILES_SECTION, Self::FILES_KEY);
    let info_portions: Option<Vec<String>> = list(Self::INFO_PORTIONS_SECTION, Self::FILES_KEY);

    let partner_files: Option<DialogPartnerFiles> =
      (characters.is_some() || profiles.is_some() || info_portions.is_some()).then(|| DialogPartnerFiles {
        characters: characters.unwrap_or_default(),
        profiles: profiles.unwrap_or_default(),
        info_portions: info_portions.unwrap_or_default(),
      });

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
      partner_files,
      profile_sections,
    }
  }
}
