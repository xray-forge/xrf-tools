use xrf_xml::XmlElementSpan;

use crate::partner::dialog_partner_text::DialogPartnerText;

/// A specific character, as far as the dialogs it offers go.
///
/// Read the way `CSpecificCharacter::load_shared` reads one, after its includes are spliced in: shipped characters take
/// most of their dialogs from `#include "gameplay\character_dialogs.xml"`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogCharacter {
  id: String,
  class: Option<String>,
  start_dialog: Option<String>,
  actor_dialogs: Vec<String>,
  is_random: bool,
}

impl DialogCharacter {
  /// The element a character description declares each character with.
  pub(crate) const ELEMENT: &'static str = "specific_character";
  const ID_ATTRIBUTE: &'static str = "id";
  const CLASS_ELEMENT: &'static str = "class";
  const START_DIALOG_ELEMENT: &'static str = "start_dialog";
  const ACTOR_DIALOG_ELEMENT: &'static str = "actor_dialog";
  const NO_RANDOM_ATTRIBUTE: &'static str = "no_random";

  /// Read one `specific_character`, or `None` for one with no id or an empty one, which the engine skips.
  pub(crate) fn read(element: &XmlElementSpan) -> Option<Self> {
    let id: &str = element.attribute(Self::ID_ATTRIBUTE).filter(|id| !id.is_empty())?;

    Some(Self {
      id: id.to_owned(),
      // The engine reads index 0 once per declared class, so only the first one ever counts.
      class: DialogPartnerText::first(element, Self::CLASS_ELEMENT).map(|class| class.to_lowercase()),
      // A shared include declares `hello_dialog` too, and the first declaration is the one the engine reads.
      start_dialog: DialogPartnerText::first(element, Self::START_DIALOG_ELEMENT),
      actor_dialogs: DialogPartnerText::all(element, Self::ACTOR_DIALOG_ELEMENT),
      is_random: element
        .attribute(Self::NO_RANDOM_ATTRIBUTE)
        .is_none_or(|value| xrf_utils::read_engine_integer(value) == 0),
    })
  }

  pub fn get_id(&self) -> &str {
    &self.id
  }

  /// The class a profile picks the character by, lowercased as the engine stores it.
  pub fn get_class(&self) -> Option<&str> {
    self.class.as_deref()
  }

  /// The dialog the character opens a talk with, saying its phrase `0`, until a script sets another.
  pub fn get_start_dialog(&self) -> Option<&str> {
    self.start_dialog.as_deref()
  }

  /// Dialogs the actor may open with the character, in declaration order.
  pub fn get_actor_dialogs(&self) -> &[String] {
    &self.actor_dialogs
  }

  /// Whether a profile naming only a class may pick the character, which `no_random` turns off.
  pub fn is_random(&self) -> bool {
    self.is_random
  }
}
