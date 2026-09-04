use std::sync::Arc;

use crate::ltx::{Ltx, Section};
use crate::syntax::ROOT_SECTION;

/// The section a lowering pass is reading into, before it is placed in the result.
///
/// Held rather than looked up. Entering [`Ltx`]'s section map for every field line cloned the section name and hashed
/// it again, which on a vanilla tree was 293,441 allocations and as many lookups to fill 23,541 sections.
pub(crate) struct LtxOpenSection {
  name: String,
  section: Section,
  /// Whether a header opened it, as opposed to a field arriving before any header did.
  is_declared: bool,
}

impl Default for LtxOpenSection {
  /// The implicit section that fields sit in before the first header.
  fn default() -> Self {
    Self {
      is_declared: false,
      name: String::from(ROOT_SECTION),
      section: Section::default(),
    }
  }
}

impl LtxOpenSection {
  /// A section a header just opened, carrying the parents that header declared.
  pub(crate) fn declared(name: &str, parents: &[Box<str>]) -> Self {
    let mut section: Section = Section::default();

    for parent in parents {
      section.inherit(&**parent);
    }

    Self {
      is_declared: true,
      name: String::from(name),
      section,
    }
  }

  /// Record one field, sharing text the interner already holds.
  pub(crate) fn insert(&mut self, key: Arc<str>, value: Arc<str>) {
    self.section.insert_shared(key, value);
  }

  /// Place this section in `ltx`, which is the end of it.
  ///
  /// A declared section lands even when it holds nothing, because the engine loaded it; an undeclared one lands only
  /// when a field reached it, so a file that opens with a header has no root section.
  pub(crate) fn close_into(self, ltx: &mut Ltx) {
    if self.is_declared || !self.section.is_empty() {
      ltx.insert_section(self.name, self.section);
    }
  }
}

#[cfg(test)]
mod test {
  use crate::dialect::LtxOpenSection;
  use crate::ltx::Ltx;
  use crate::syntax::ROOT_SECTION;

  #[test]
  fn an_untouched_root_section_lands_nowhere() {
    let mut ltx: Ltx = Ltx::new();

    LtxOpenSection::default().close_into(&mut ltx);

    assert!(ltx.is_empty(), "a file that declares nothing to hold no sections");
  }

  #[test]
  fn a_root_section_a_field_reached_lands() {
    let mut ltx: Ltx = Ltx::new();
    let mut open: LtxOpenSection = LtxOpenSection::default();

    open.insert("key".into(), "value".into());
    open.close_into(&mut ltx);

    assert_eq!(ltx.get_from(ROOT_SECTION, "key"), Some("value"));
  }

  #[test]
  fn a_declared_section_lands_even_when_it_holds_nothing() {
    let mut ltx: Ltx = Ltx::new();

    LtxOpenSection::declared("empty", &[]).close_into(&mut ltx);

    assert!(ltx.has_section("empty"));
    assert_eq!(ltx.section("empty").expect("the section").len(), 0);
  }

  #[test]
  fn a_declared_section_carries_the_parents_its_header_named() {
    let mut ltx: Ltx = Ltx::new();

    LtxOpenSection::declared("child", &[Box::from("first"), Box::from("second")]).close_into(&mut ltx);

    let section: &crate::ltx::Section = ltx.section("child").expect("the section");

    assert!(section.inherits_section("first"));
    assert!(section.inherits_section("second"));
  }
}
