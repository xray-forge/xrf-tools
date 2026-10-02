use std::collections::HashSet;

use indexmap::IndexMap;
use indexmap::map::Entry;
use xrf_error::XrfResult;
use xrf_extension::XrayExtension;
use xrf_vfs::{XrayAsset, XrayLogicalPath, XrayLookupScope, XrayVfs};
use xrf_xml::{XmlElementSpan, XmlParseOptions, XmlSourceDocument, expand_xml_includes, list_xml_includes};

use crate::constants::ID_ATTRIBUTE;
use crate::encoding::decode;
use crate::partner::dialog_character::DialogCharacter;
use crate::partner::dialog_offer::DialogOffer;
use crate::partner::dialog_profile::DialogProfile;
use crate::project::descriptor::DialogFinding;

/// Reads one partner file's root into the partners, given the file's logical path for findings.
type PartnerReader = fn(&mut DialogPartners, &XmlElementSpan, &str);

/// The talk partners a game tree declares: characters, the profiles naming them, and the info portions opening dialogs.
///
/// Answers which dialogs reach the actor when it talks to an NPC, the way `CActor::UpdateAvailableDialogs` gathers them.
/// A file that cannot be read becomes a finding, as it does for `DialogProject`.
#[derive(Debug, Default)]
pub struct DialogPartners {
  characters: IndexMap<String, DialogCharacter>,
  profiles: IndexMap<String, DialogProfile>,
  info_dialogs: IndexMap<String, Vec<String>>,
  findings: Vec<DialogFinding>,
}

impl DialogPartners {
  /// The finding a file that could not be read becomes.
  pub const UNREADABLE_RULE: &'static str = "dialog.partner-unreadable";
  /// The finding a second declaration of an id becomes.
  pub const DUPLICATE_RULE: &'static str = "dialog.partner-duplicate";

  // todo: Read the files `system.ltx` lists under `[profiles]` and `[info_portions]` instead of matching file names.
  const CHARACTERS_FILE_PREFIX: &'static str = "character_desc";
  const PROFILES_FILE_PREFIX: &'static str = "npc_profile";
  const INFO_PORTIONS_FILE_PREFIX: &'static str = "info_";

  const INFO_PORTION_ELEMENT: &'static str = "info_portion";
  const INFO_PORTION_DIALOG_ELEMENT: &'static str = "dialog";

  /// The prefix the engine opens XML includes under, `$game_config$`.
  const INCLUDES_PREFIX: &'static str = "configs";

  /// Read every character description, profile list and info portion file under a prefix of mounted roots.
  ///
  /// # Errors
  ///
  /// Returns an error when the prefix is not a valid logical path.
  pub fn from_vfs(vfs: &XrayVfs, prefix: &str) -> XrfResult<Self> {
    let scope: XrayLookupScope = XrayLookupScope::all().with_prefix(prefix)?;
    let mut assets: Vec<XrayAsset> = vfs
      .scoped(&scope)
      .list_entries()
      .into_iter()
      .filter(|asset| asset.get_logical_path().has_extension(XrayExtension::Xml))
      .collect();

    // Mount order is not name order, and the first of two declarations is the one kept.
    assets.sort_by(|left, right| left.get_logical_path().as_str().cmp(right.get_logical_path().as_str()));

    let sources: Vec<(&str, PartnerReader, XrfResult<Vec<u8>>)> = assets
      .iter()
      .filter_map(|asset| {
        let logical_path: &str = asset.get_logical_path().as_str();

        Self::find_reader(asset.get_logical_path().file_name())
          .map(|reader| (logical_path, reader, vfs.read_bytes(logical_path)))
      })
      .collect();

    // A file another one includes is part of that file. The engine never loads it on its own, and Anomaly writes such
    // parts without a root, so reading one by itself would only report it unreadable. Named from the text, so a part
    // still counts when the file including it fails on an earlier include.
    let included: HashSet<String> = sources
      .iter()
      .filter_map(|(_, _, source)| source.as_ref().ok())
      .flat_map(|source| list_xml_includes(source))
      .filter_map(|name| XrayLogicalPath::normalize(&Self::to_include_path(&name)).ok())
      .collect();

    let mut partners: Self = Self::default();

    for (logical_path, reader, source) in sources {
      if included.contains(logical_path) {
        continue;
      }

      let document: XrfResult<XmlSourceDocument> = source
        .and_then(|source| expand_xml_includes(&source, &mut |name| vfs.read_bytes(&Self::to_include_path(name))))
        .and_then(|expanded| decode(&expanded))
        .and_then(|decoded| XmlSourceDocument::parse(decoded.text, XmlParseOptions::default()));

      match document {
        Ok(document) => reader(&mut partners, document.root(), logical_path),
        Err(error) => partners.findings.push(DialogFinding::new(
          Self::UNREADABLE_RULE,
          Some(logical_path.to_owned()),
          error.to_string(),
        )),
      }
    }

    Ok(partners)
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
  pub fn get_info_dialogs(&self) -> &IndexMap<String, Vec<String>> {
    &self.info_dialogs
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

    for (info, dialogs) in &self.info_dialogs {
      for dialog in dialogs {
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

  /// The logical path of an included file, which the engine opens under `$game_config$`.
  fn to_include_path(name: &str) -> String {
    format!("{}\\{name}", Self::INCLUDES_PREFIX)
  }

  fn read_characters(&mut self, root: &XmlElementSpan, logical_path: &str) {
    for character in root
      .children_named(DialogCharacter::ELEMENT)
      .filter_map(DialogCharacter::read)
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
      .filter_map(DialogProfile::read)
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
    for element in root.children_named(Self::INFO_PORTION_ELEMENT) {
      let Some(id) = element.attribute(ID_ATTRIBUTE) else {
        continue;
      };

      let dialogs: Vec<String> = element
        .children_named(Self::INFO_PORTION_DIALOG_ELEMENT)
        .map(|child| child.text().trim().to_owned())
        .filter(|dialog| !dialog.is_empty())
        .collect();

      // Kept whether it opens anything or not, so a later declaration of the id is a duplicate either way.
      Self::insert(
        &mut self.info_dialogs,
        &mut self.findings,
        "Info portion",
        id.to_owned(),
        dialogs,
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
