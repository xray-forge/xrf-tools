use crate::element::{DialogElement, DialogElementKind};

/// Build an element with a throwaway range, for the classification tests.
fn element(name: &str, value: &str) -> DialogElement {
  DialogElement::new(String::from(name), String::from(value), 0..0)
}

#[test]
fn classifies_every_element_the_shipped_files_use() {
  for (name, kind) in [
    ("text", DialogElementKind::Text),
    ("script_text", DialogElementKind::ScriptText),
    ("action", DialogElementKind::Action),
    ("precondition", DialogElementKind::Precondition),
    ("next", DialogElementKind::Next),
    ("give_info", DialogElementKind::GiveInfo),
    ("disable_info", DialogElementKind::DisableInfo),
    ("has_info", DialogElementKind::HasInfo),
    ("dont_has_info", DialogElementKind::DontHasInfo),
    ("is_final", DialogElementKind::IsFinal),
    ("init_func", DialogElementKind::InitFunc),
    ("phrase_list", DialogElementKind::Container),
    ("phrase", DialogElementKind::Container),
  ] {
    assert_eq!(DialogElementKind::from_name(name), kind, "for '{name}'");
  }
}

#[test]
fn treats_an_element_it_does_not_know_as_unknown() {
  // A real one: a shipped project uses `go_back` on phrases that return to the dialog root.
  assert_eq!(DialogElementKind::from_name("go_back"), DialogElementKind::Unknown);
  assert_eq!(DialogElementKind::from_name(""), DialogElementKind::Unknown);
}

#[test]
fn separates_the_dialog_and_phrase_element_sets() {
  // `action` is valid at both levels; the shipped files use it twice on a dialog.
  assert!(DialogElementKind::Action.is_valid_for_dialog());
  assert!(DialogElementKind::Action.is_valid_for_phrase());

  assert!(DialogElementKind::InitFunc.is_valid_for_dialog());
  assert!(!DialogElementKind::InitFunc.is_valid_for_phrase());

  assert!(DialogElementKind::Next.is_valid_for_phrase());
  assert!(!DialogElementKind::Next.is_valid_for_dialog());

  assert!(DialogElementKind::Text.is_valid_for_phrase());
  assert!(!DialogElementKind::Text.is_valid_for_dialog());

  assert!(!DialogElementKind::Unknown.is_valid_for_dialog());
  assert!(!DialogElementKind::Unknown.is_valid_for_phrase());
}

#[test]
fn keeps_the_name_it_was_written_with() {
  // Classification is a view; a rewrite has to reproduce the original name, including an unknown one.
  let unknown: DialogElement = element("go_back", "1");

  assert_eq!(unknown.get_name(), "go_back");
  assert_eq!(unknown.get_kind(), DialogElementKind::Unknown);
}

#[test]
fn reads_a_flag_the_way_the_engine_does() {
  assert!(element("is_final", "1").is_enabled());
  assert!(element("is_final", "true").is_enabled());
  // Whitespace is what an element written across lines carries.
  assert!(!element("is_final", "  ").is_enabled());
  assert!(!element("is_final", "").is_enabled());
  assert!(!element("is_final", "0").is_enabled());
}
