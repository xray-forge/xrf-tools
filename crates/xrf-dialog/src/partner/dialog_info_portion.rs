use xrf_xml::XmlElementSpan;

use crate::partner::dialog_partner_text::DialogPartnerText;

/// An info portion as its declaration states it: the dialogs it opens, which reach the actor from every NPC while the
/// actor holds it, and the portions giving it takes away.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogInfoPortion {
  id: String,
  logical_path: String,
  dialogs: Vec<String>,
  disables: Vec<String>,
}

impl DialogInfoPortion {
  /// The element an info portion file declares each info portion with.
  pub(crate) const ELEMENT: &'static str = "info_portion";
  const ID_ATTRIBUTE: &'static str = "id";
  const DIALOG_ELEMENT: &'static str = "dialog";
  const DISABLE_ELEMENT: &'static str = "disable";

  /// Read one `info_portion` of the file at a logical path, or `None` for one with no id.
  pub(crate) fn read(element: &XmlElementSpan, logical_path: &str) -> Option<Self> {
    let id: &str = element.attribute(Self::ID_ATTRIBUTE)?;

    Some(Self {
      id: id.to_owned(),
      logical_path: logical_path.to_owned(),
      dialogs: DialogPartnerText::all(element, Self::DIALOG_ELEMENT),
      disables: DialogPartnerText::all(element, Self::DISABLE_ELEMENT),
    })
  }

  pub fn get_id(&self) -> &str {
    &self.id
  }

  /// The logical path of the file declaring the info portion.
  pub fn get_logical_path(&self) -> &str {
    &self.logical_path
  }

  /// The dialogs the info portion opens, in declaration order; most open none.
  pub fn get_dialogs(&self) -> &[String] {
    &self.dialogs
  }

  /// The info portions giving this one takes away (`<disable>`), in declaration order; most take none.
  pub fn get_disables(&self) -> &[String] {
    &self.disables
  }
}
