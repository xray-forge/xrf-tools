use serde::Serialize;
use xrf_ltx::fold_section_name;

/// What a declaration search asks for: a section, a field, a value, or any combination.
///
/// Names match whole, as `LtxRootReader::read_sections` resolves them. Field names are case-sensitive; section names
/// ignore case under [`Self::is_section_case_folded`]. A value matches as a case-sensitive substring of the text after
/// `=`, since a value is usually a list or condlist and a person remembers one item of it.
///
/// Headers answer only a query naming a section alone; a field or value query answers field lines. A query naming
/// nothing matches nothing.
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
  /// Whether section names compare without regard to case, as the dialect folds them
  /// (`LtxDialect::is_folding_section_names`, true for both shipped dialects).
  pub is_section_case_folded: bool,
}

impl LtxDeclarationQuery {
  /// Whether one section header answers this query.
  pub fn matches_header(&self, section: &str) -> bool {
    self.field.is_none() && self.value.is_none() && self.matches_section_name(section)
  }

  /// Whether one field line answers this query.
  ///
  /// `section` is the header above the line; a query naming a section never matches a key before the first header. A
  /// bare key never answers a value query.
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

  /// Whether a header's name is the section this query names, compared as the dialect resolves it.
  pub fn matches_section_name(&self, name: &str) -> bool {
    match self.section.as_deref() {
      Some(wanted) if self.is_section_case_folded => fold_section_name(wanted) == fold_section_name(name),
      Some(wanted) => wanted == name,
      None => false,
    }
  }
}
