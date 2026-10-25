use xrf_xml::XmlElementSpan;

/// The text values a partner file writes as child elements.
///
/// Trimmed, and an empty one is no value at all, which is how every partner reader treats a class, a character or a
/// dialog name.
pub(crate) struct DialogPartnerText;

impl DialogPartnerText {
  /// The first child of this name's value, which is the one the engine reads for a single-valued field.
  pub(crate) fn first(element: &XmlElementSpan, name: &str) -> Option<String> {
    element
      .child_named(name)
      .map(|child| child.text().trim().to_owned())
      .filter(|value| !value.is_empty())
  }

  /// Every child of this name's value, in declaration order.
  pub(crate) fn all(element: &XmlElementSpan, name: &str) -> Vec<String> {
    element
      .children_named(name)
      .map(|child| child.text().trim().to_owned())
      .filter(|value| !value.is_empty())
      .collect()
  }
}
