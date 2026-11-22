use xrf_xml::XmlElementSpan;

use crate::partner::dialog_partner_text::DialogPartnerText;

/// An info portion, as far as the dialogs it opens go: while the actor holds it, each of them reaches the actor from
/// every NPC.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogInfoPortion {
  id: String,
  logical_path: String,
  dialogs: Vec<String>,
}

impl DialogInfoPortion {
  /// The element an info portion file declares each info portion with.
  pub(crate) const ELEMENT: &'static str = "info_portion";
  const ID_ATTRIBUTE: &'static str = "id";
  const DIALOG_ELEMENT: &'static str = "dialog";

  /// Read one `info_portion` of the file at a logical path, or `None` for one with no id.
  pub(crate) fn read(element: &XmlElementSpan, logical_path: &str) -> Option<Self> {
    let id: &str = element.attribute(Self::ID_ATTRIBUTE)?;

    Some(Self {
      id: id.to_owned(),
      logical_path: logical_path.to_owned(),
      dialogs: DialogPartnerText::all(element, Self::DIALOG_ELEMENT),
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
}
