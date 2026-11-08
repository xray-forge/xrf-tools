use serde::Serialize;

/// What a declaration search asks for: a section, a field, a value, or any combination.
///
/// Names match whole, the way resolution looks them up, so a section found here is the section
/// `LtxRootReader::read_sections` resolves under the same name. A field name is compared case-sensitively; a section
/// name too, unless `is_section_case_folded` says the dialect lowercases section names, as DLTX does
/// (`LtxDialect::is_folding_section_names`). A value matches as a case-sensitive substring of the text written after
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
  /// Whether section names compare without regard to case, because the dialect resolving them folds it.
  pub is_section_case_folded: bool,
}

impl LtxDeclarationQuery {
  /// Whether one section header answers this query.
  pub fn matches_header(&self, section: &str) -> bool {
    self.field.is_none() && self.value.is_none() && self.matches_section_name(section)
  }

  /// Whether one field line answers this query.
  ///
  /// `section` is the header above the line, absent for a key written before the first header; a query naming a
  /// section never matches such a key. A bare key spells no value, so it never answers a value query.
  pub fn matches_field(&self, section: Option<&str>, field: &str, value: Option<&str>) -> bool {
    if self.field.is_none() && self.value.is_none() {
      return false;
    }

    if self.section.is_some() && !section.is_some_and(|section| self.matches_section_name(section)) {
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

  /// Whether a header's name is the section this query names, compared as the dialect resolves names.
  ///
  /// Lowercased whole, as `to_lowercase` does and as the DLTX loader does, rather than ASCII-only: the two must agree
  /// on every name or a header the resolver merges would go unfound.
  pub fn matches_section_name(&self, name: &str) -> bool {
    match self.section.as_deref() {
      Some(wanted) if self.is_section_case_folded => wanted.to_lowercase() == name.to_lowercase(),
      Some(wanted) => wanted == name,
      None => false,
    }
  }
}
