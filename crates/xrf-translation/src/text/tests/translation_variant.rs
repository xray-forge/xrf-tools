use crate::text::translation_variant::TranslationVariant;

#[test]
fn a_single_string_renders_as_itself() {
  assert_eq!(
    TranslationVariant::String(String::from("one line")).to_single_line(),
    "one line"
  );
}

#[test]
fn a_multi_line_value_joins_on_the_engine_line_break() {
  assert_eq!(
    TranslationVariant::MultiString(vec![String::from("first"), String::from("second")]).to_single_line(),
    "first\\nsecond"
  );
}

#[test]
fn table_text_splits_on_the_line_break_it_joins_on() {
  let read: TranslationVariant = TranslationVariant::from_table_text("first\\nsecond");

  assert_eq!(
    read,
    TranslationVariant::MultiString(vec![String::from("first"), String::from("second")])
  );
  assert_eq!(read.to_single_line(), "first\\nsecond");
  assert_eq!(
    TranslationVariant::from_table_text("one line"),
    TranslationVariant::String(String::from("one line"))
  );
}
