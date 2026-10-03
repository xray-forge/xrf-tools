use xrf_xml::XmlElementSpan;

use crate::constants::ID_ATTRIBUTE;

/// An NPC profile: the specific character an NPC is spawned as, or the class one is picked from.
///
/// Read the way `CCharacterInfo::load_shared` reads one. A pinned character makes the class irrelevant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogProfile {
  id: String,
  specific_character: Option<String>,
  class: Option<String>,
}

impl DialogProfile {
  /// The element a profile list declares each profile with.
  pub(crate) const ELEMENT: &'static str = "character";
  const SPECIFIC_CHARACTER_ELEMENT: &'static str = "specific_character";
  const CLASS_ELEMENT: &'static str = "class";

  /// Read one profile `character`, or `None` for one with no id, which no NPC can be spawned with.
  pub(crate) fn read(element: &XmlElementSpan) -> Option<Self> {
    let id: &str = element.attribute(ID_ATTRIBUTE)?;

    Some(Self {
      id: id.to_owned(),
      specific_character: Self::read_first(element, Self::SPECIFIC_CHARACTER_ELEMENT),
      class: Self::read_first(element, Self::CLASS_ELEMENT).map(|class| class.to_lowercase()),
    })
  }

  /// The id `profile_name()` answers for an NPC spawned with this profile.
  pub fn get_id(&self) -> &str {
    &self.id
  }

  /// The character every NPC of this profile is, when the profile pins one.
  pub fn get_specific_character(&self) -> Option<&str> {
    self.specific_character.as_deref()
  }

  /// The class a character is picked from otherwise. `None` admits every character a profile may pick.
  pub fn get_class(&self) -> Option<&str> {
    self.class.as_deref()
  }

  fn read_first(element: &XmlElementSpan, name: &str) -> Option<String> {
    element
      .child_named(name)
      .map(|child| child.text().trim().to_owned())
      .filter(|value| !value.is_empty())
  }
}
