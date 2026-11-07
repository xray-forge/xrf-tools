use serde::Serialize;

/// What a declaration search asks for: a section, a field, a value, or any combination.
///
/// Names match whole and case-sensitively, the way resolution looks them up, so a section found here is the section
/// `LtxRootReader::read_sections` resolves under the same name. A value matches as a case-sensitive substring of the text written after
/// `=`, because a value is usually a list or a condlist and the part a person remembers is one item of it.
///
/// A header is answered only when the query names a section and nothing narrower; once a field or value is asked for,
/// the answer is the field lines alone. A query naming nothing matches nothing.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxDeclarationQuery {
  /// Section name without its brackets or operation prefix.
  pub section: Option<String>,
  /// Field name without its operation prefix.
  pub field: Option<String>,
  /// Text the written value must contain.
  pub value: Option<String>,
}

impl LtxDeclarationQuery {
  /// Whether one section header answers this query.
  pub fn matches_header(&self, section: &str) -> bool {
    self.field.is_none() && self.value.is_none() && self.section.as_deref() == Some(section)
  }

  /// Whether one field line answers this query.
  ///
  /// `section` is the header above the line, absent for a key written before the first header; a query naming a
  /// section never matches such a key. A bare key spells no value, so it never answers a value query.
  pub fn matches_field(&self, section: Option<&str>, field: &str, value: Option<&str>) -> bool {
    if self.field.is_none() && self.value.is_none() {
      return false;
    }

    if self.section.as_deref().is_some_and(|wanted| section != Some(wanted)) {
      return false;
    }

    if self.field.as_deref().is_some_and(|wanted| wanted != field) {
      return false;
    }

    match self.value.as_deref() {
      Some(wanted) => value.is_some_and(|written| written.contains(wanted)),
      None => true,
    }
  }
}
