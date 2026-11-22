use std::collections::HashSet;

use indexmap::IndexMap;
use indexmap::map::Entry;
use xrf_error::XrfResult;
use xrf_extension::XrayExtension;
use xrf_vfs::{XrayLogicalPath, XraySearch};
use xrf_xml::{XmlElementSpan, XmlEngineSource, XmlSourceDocument};

use crate::dialog_listed_files::DialogListedFiles;
use crate::partner::dialog_character::DialogCharacter;
use crate::partner::dialog_info_portion::DialogInfoPortion;
use crate::partner::dialog_offer::DialogOffer;
use crate::partner::dialog_profile::DialogProfile;
use crate::project::dialog_finding::DialogFinding;

/// Reads one partner file's root into the partners, given the file's logical path for findings.
type PartnerReader = fn(&mut DialogPartners, &XmlElementSpan, &str);

/// A file whose name marks it as holding partners, with the reader its name picks.
struct NamedPartnerFile {
  logical_path: String,
  reader: PartnerReader,
}

/// The talk partners a game tree declares: characters, the profiles naming them, and the info portions opening dialogs.
///
/// Answers which dialogs reach the actor when it talks to an NPC, the way `CActor::UpdateAvailableDialogs` gathers
/// them. A file that cannot be read becomes a finding, as it does for `DialogProject`.
#[derive(Debug, Default)]
pub struct DialogPartners {
  characters: IndexMap<String, DialogCharacter>,
  profiles: IndexMap<String, DialogProfile>,
  info_portions: IndexMap<String, DialogInfoPortion>,
  findings: Vec<DialogFinding>,
}

impl DialogPartners {
  /// The finding a file that could not be read becomes.
  pub const UNREADABLE_RULE: &'static str = "dialog.partner-unreadable";
  /// The finding a listed file the tree does not hold becomes.
  pub const MISSING_RULE: &'static str = "dialog.partner-missing";
  /// The finding a second declaration of an id becomes.
  pub const DUPLICATE_RULE: &'static str = "dialog.partner-duplicate";

  // todo: Have `dialog list` read the files `system.ltx` lists through `from_listed` instead of matching file names.
  const CHARACTERS_FILE_PREFIX: &'static str = "character_desc";
  const PROFILES_FILE_PREFIX: &'static str = "npc_profile";
  const INFO_PORTIONS_FILE_PREFIX: &'static str = "info_";

  /// Read exactly the partner files `system.ltx` lists under a prefix, in list order, as the engine's id index does.
  ///
  /// A listed file the tree does not hold, or one that does not read, becomes a finding.
  pub fn from_listed(search: &XraySearch, prefix: &str, files: &DialogListedFiles) -> Self {
    let mut partners: Self = Self::default();

    partners.read_listed(search, prefix, &files.characters, Self::read_characters);
    partners.read_listed(search, prefix, &files.profiles, Self::read_profiles);
    partners.read_listed(search, prefix, &files.info_portions, Self::read_info_portions);

    partners
  }

  /// Read the files of one list, each a name under the prefix without its extension.
  fn read_listed(&mut self, search: &XraySearch, prefix: &str, names: &[String], reader: PartnerReader) {
    for name in names {
      let logical_path: String = DialogListedFiles::resolve_logical_path(prefix, name);

      if matches!(search.find(&logical_path), Ok(None)) {
        self.findings.push(DialogFinding::new(
          Self::MISSING_RULE,
          Some(logical_path),
          format!("'{name}' is listed in system.ltx, but the tree holds no such file"),
        ));

        continue;
      }

      self.read_file(search, &logical_path, search.read(&logical_path), reader);
    }
  }

  /// Read every character description, profile list and info portion file a search reaches under a prefix, choosing
  /// them by name, for a tree whose `system.ltx` lists none.
  ///
  /// # Errors
  ///
  /// Returns an error when the prefix is not a valid logical path.
  pub fn from_search(search: &XraySearch, prefix: &str) -> XrfResult<Self> {
    let mut partners: Self = Self::default();

    for file in Self::list_named(search, prefix)? {
      partners.read_file(search, &file.logical_path, search.read(&file.logical_path), file.reader);
    }

    Ok(partners)
  }

  /// Every file under a prefix whose name marks it a character description, profile list or info portion file, in
  /// engine-identity order, leaving out the parts another such file includes.
  ///
  /// These are the files a tree would declare partners in were every one of them listed, so a caller holding the
  /// `system.ltx` lists can tell which ones the engine never loads.
  ///
  /// # Errors
  ///
  /// Returns an error when the prefix is not a valid logical path.
  pub fn list_named_files(search: &XraySearch, prefix: &str) -> XrfResult<Vec<String>> {
    Ok(
      Self::list_named(search, prefix)?
        .into_iter()
        .map(|file| file.logical_path)
        .collect(),
    )
  }

  fn list_named(search: &XraySearch, prefix: &str) -> XrfResult<Vec<NamedPartnerFile>> {
    // In engine-identity order rather than mount order, since the first of two declarations is the one kept.
    let named: Vec<NamedPartnerFile> = search
      .clone()
      .with_prefix(prefix)?
      .list_files()
      .into_iter()
      .filter(|asset| asset.get_logical_path().has_extension(XrayExtension::Xml))
      .filter_map(|asset| {
        Self::find_reader(asset.get_logical_path().file_name()).map(|reader| NamedPartnerFile {
          logical_path: asset.get_logical_path().as_str().to_owned(),
          reader,
        })
      })
      .collect();

    // A part another file includes is read through that file; Anomaly writes such parts without a root.
    let sources: Vec<Vec<u8>> = named
      .iter()
      .filter_map(|file| search.read(&file.logical_path).ok())
      .collect();
    let included: HashSet<String> = XmlEngineSource::list_included(sources.iter().map(Vec::as_slice))
      .into_iter()
      .filter_map(|path| XrayLogicalPath::normalize(&path).ok())
      .collect();

    Ok(
      named
        .into_iter()
        .filter(|file| !included.contains(&file.logical_path))
        .collect(),
    )
  }

  /// Parse one partner file with its includes spliced in and hand its root to a reader, or record why it did not read.
  fn read_file(&mut self, search: &XraySearch, logical_path: &str, source: XrfResult<Vec<u8>>, reader: PartnerReader) {
    let document: XrfResult<XmlSourceDocument> =
      source.and_then(|source| XmlEngineSource::parse(&source, &mut |path| search.read(path)));

    match document {
      Ok(document) => reader(self, document.root(), logical_path),
      Err(error) => self.findings.push(DialogFinding::new(
        Self::UNREADABLE_RULE,
        Some(logical_path.to_owned()),
        error.to_string(),
      )),
    }
  }

  /// Specific characters by id, in the order their files were read.
  pub fn get_characters(&self) -> &IndexMap<String, DialogCharacter> {
    &self.characters
  }

  /// Profiles by id, in the order their files were read.
  pub fn get_profiles(&self) -> &IndexMap<String, DialogProfile> {
    &self.profiles
  }

  /// Every info portion by id, with the dialogs it opens with every NPC, which for most is none.
  pub fn get_info_portions(&self) -> &IndexMap<String, DialogInfoPortion> {
    &self.info_portions
  }

  pub fn get_findings(&self) -> &[DialogFinding] {
    &self.findings
  }

  pub fn find_profile(&self, id: &str) -> Option<&DialogProfile> {
    self.profiles.get(id)
  }

  /// The characters an NPC of a profile can be: the pinned one, or every character the profile may pick by class.
  ///
  /// The engine picks one of the candidates at spawn and narrows them by rank and reputation first when it can, so this
  /// answers every character the NPC may be.
  pub fn resolve_profile(&self, profile: &DialogProfile) -> Vec<&DialogCharacter> {
    if let Some(id) = profile.get_specific_character() {
      return self.characters.get(id).into_iter().collect();
    }

    self
      .characters
      .values()
      .filter(|character| character.is_random())
      .filter(|character| {
        profile
          .get_class()
          .is_none_or(|class| character.get_class() == Some(class))
      })
      .collect()
  }

  /// Whether an NPC of a profile can be spawned at all, as `CSE_ALifeTraderAbstract::specific_character` decides.
  ///
  /// A pinned character has to be declared. A profile picking by class falls back to a `team_default` character of the
  /// class when rank, reputation and characters already in use rule the others out, and the engine asserts on a class
  /// without one, so at least one has to be there whether or not the fallback is ever taken.
  pub fn is_profile_resolvable(&self, profile: &DialogProfile) -> bool {
    if profile.get_specific_character().is_some() {
      return !self.resolve_profile(profile).is_empty();
    }

    self
      .resolve_profile(profile)
      .iter()
      .any(|character| character.is_team_default())
  }

  /// Every dialog that reaches the actor when it talks to an NPC of a profile, with each way it does.
  ///
  /// Start dialogs come first, then the characters' actor dialogs, then the dialogs of info portions, each in
  /// declaration order. A dialog reached two ways lists both, since either one offers it.
  pub fn list_offers(&self, profile: &DialogProfile) -> IndexMap<String, Vec<DialogOffer>> {
    let characters: Vec<&DialogCharacter> = self.resolve_profile(profile);
    let mut offers: IndexMap<String, Vec<DialogOffer>> = IndexMap::new();

    for character in &characters {
      if let Some(dialog) = character.get_start_dialog() {
        offers.entry(dialog.to_owned()).or_default().push(DialogOffer::Start {
          character: character.get_id().to_owned(),
        });
      }
    }

    for character in &characters {
      for dialog in character.get_actor_dialogs() {
        offers.entry(dialog.clone()).or_default().push(DialogOffer::Actor {
          character: character.get_id().to_owned(),
        });
      }
    }

    for (info, portion) in &self.info_portions {
      for dialog in portion.get_dialogs() {
        offers
          .entry(dialog.clone())
          .or_default()
          .push(DialogOffer::Info { info: info.clone() });
      }
    }

    offers
  }

  /// The reader for a file, by its name, or `None` for a file holding no partners.
  fn find_reader(file_name: &str) -> Option<PartnerReader> {
    if file_name.starts_with(Self::CHARACTERS_FILE_PREFIX) {
      Some(Self::read_characters)
    } else if file_name.starts_with(Self::PROFILES_FILE_PREFIX) {
      Some(Self::read_profiles)
    } else if file_name.starts_with(Self::INFO_PORTIONS_FILE_PREFIX) {
      Some(Self::read_info_portions)
    } else {
      None
    }
  }

  fn read_characters(&mut self, root: &XmlElementSpan, logical_path: &str) {
    for character in root
      .children_named(DialogCharacter::ELEMENT)
      .filter_map(|element| DialogCharacter::read(element, logical_path))
    {
      let id: String = character.get_id().to_owned();

      Self::insert(
        &mut self.characters,
        &mut self.findings,
        "Character",
        id,
        character,
        logical_path,
      );
    }
  }

  fn read_profiles(&mut self, root: &XmlElementSpan, logical_path: &str) {
    for profile in root
      .children_named(DialogProfile::ELEMENT)
      .filter_map(|element| DialogProfile::read(element, logical_path))
    {
      let id: String = profile.get_id().to_owned();

      Self::insert(
        &mut self.profiles,
        &mut self.findings,
        "Profile",
        id,
        profile,
        logical_path,
      );
    }
  }

  fn read_info_portions(&mut self, root: &XmlElementSpan, logical_path: &str) {
    for portion in root
      .children_named(DialogInfoPortion::ELEMENT)
      .filter_map(|element| DialogInfoPortion::read(element, logical_path))
    {
      let id: String = portion.get_id().to_owned();

      // Kept whether it opens anything or not, so a later declaration of the id is a duplicate either way.
      Self::insert(
        &mut self.info_portions,
        &mut self.findings,
        "Info portion",
        id,
        portion,
        logical_path,
      );
    }
  }

  /// Keep the first declaration of an id, as the engine's id index does, and report the others.
  fn insert<T>(
    map: &mut IndexMap<String, T>,
    findings: &mut Vec<DialogFinding>,
    kind: &str,
    id: String,
    value: T,
    logical_path: &str,
  ) {
    match map.entry(id) {
      Entry::Vacant(entry) => {
        entry.insert(value);
      }
      Entry::Occupied(entry) => findings.push(DialogFinding::new(
        Self::DUPLICATE_RULE,
        Some(logical_path.to_owned()),
        format!(
          "{kind} '{}' is declared again, and the first declaration is the one the engine reads",
          entry.key()
        ),
      )),
    }
  }
}
