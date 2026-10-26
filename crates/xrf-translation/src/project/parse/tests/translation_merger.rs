use crate::project::parse::translation_merger::TranslationMerger;
use crate::text::translation_json::TranslationJson;
use crate::text::translation_variant::TranslationVariant;

fn document(source: &str) -> TranslationJson {
  serde_json::from_str(source).expect("Expected valid test JSON")
}

#[test]
fn an_absent_id_is_inserted() {
  let (merged, outcome) = TranslationMerger::merge(
    TranslationJson::default(),
    &[(String::from("st_a"), String::from("A"))],
    "eng",
    false,
  );

  assert_eq!(outcome.inserted, 1);
  assert_eq!(
    merged["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("A")))
  );
}

#[test]
fn a_null_placeholder_is_filled() {
  let (merged, outcome) = TranslationMerger::merge(
    document(r#"{"st_a":{"eng":null}}"#),
    &[(String::from("st_a"), String::from("A"))],
    "eng",
    false,
  );

  assert_eq!(outcome.filled, 1);
  assert_eq!(outcome.inserted, 0);
  assert_eq!(
    merged["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("A")))
  );
}

#[test]
fn existing_text_survives_a_conflict_unless_overwrite_is_asked_for() {
  let kept = TranslationMerger::merge(
    document(r#"{"st_a":{"eng":"Mine"}}"#),
    &[(String::from("st_a"), String::from("Theirs"))],
    "eng",
    false,
  );

  assert_eq!(kept.1.conflicted, 1);
  assert_eq!(
    kept.0["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("Mine")))
  );

  let replaced = TranslationMerger::merge(
    document(r#"{"st_a":{"eng":"Mine"}}"#),
    &[(String::from("st_a"), String::from("Theirs"))],
    "eng",
    true,
  );

  assert_eq!(replaced.1.conflicted, 1);
  assert_eq!(
    replaced.0["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("Theirs")))
  );
}

#[test]
fn an_array_and_the_string_it_joins_to_are_the_same_text() {
  // Re-importing what a previous run wrote must not read as a conflict, or every multi-line description in the tree
  // would report one on every run.
  let (_, outcome) = TranslationMerger::merge(
    document(r#"{"st_a":{"eng":["first","second"]}}"#),
    &[(String::from("st_a"), String::from("first\\nsecond"))],
    "eng",
    false,
  );

  assert_eq!(outcome.unchanged, 1);
  assert_eq!(outcome.conflicted, 0);
}

#[test]
fn multi_line_text_is_split_into_an_array() {
  let (merged, _) = TranslationMerger::merge(
    TranslationJson::default(),
    &[(String::from("st_a"), String::from("first\\nsecond"))],
    "eng",
    false,
  );

  assert_eq!(
    merged["st_a"]["eng"],
    Some(TranslationVariant::MultiString(vec![
      String::from("first"),
      String::from("second")
    ]))
  );
}

#[test]
fn every_record_gains_a_placeholder_for_the_languages_the_file_carries() {
  // `st_b` has no Russian, and the file has Russian, so the gap becomes visible rather than absent.
  let (merged, outcome) = TranslationMerger::merge(
    document(r#"{"st_a":{"eng":"A","rus":"А"}}"#),
    &[(String::from("st_b"), String::from("B"))],
    "eng",
    false,
  );

  assert_eq!(outcome.placeholders_added, 1);
  assert!(merged["st_b"].contains_key("rus"));
  assert_eq!(merged["st_b"]["rus"], None);
  // No language the file does not carry is invented.
  assert!(!merged["st_b"].contains_key("ukr"));
}

#[test]
fn output_does_not_depend_on_the_order_the_languages_were_run_in() {
  let english = &[(String::from("st_b"), String::from("B"))];
  let russian = &[(String::from("st_a"), String::from("А"))];

  let (first, _) = TranslationMerger::merge(TranslationJson::default(), english, "eng", false);
  let (english_then_russian, _) = TranslationMerger::merge(first, russian, "rus", false);

  let (second, _) = TranslationMerger::merge(TranslationJson::default(), russian, "rus", false);
  let (russian_then_english, _) = TranslationMerger::merge(second, english, "eng", false);

  assert_eq!(
    serde_json::to_string(&english_then_russian).unwrap(),
    serde_json::to_string(&russian_then_english).unwrap()
  );
}

#[test]
fn a_hand_edited_file_is_sorted_even_when_nothing_merges_into_it() {
  // A tree somebody added records to by hand is normalized by the next run that touches the file, even when that run
  // merges nothing new into it.
  let (merged, _) = TranslationMerger::merge(
    document(r#"{"st_b":{"eng":"B"},"st_a":{"eng":"A"}}"#),
    &[],
    "eng",
    false,
  );

  assert_eq!(merged.keys().collect::<Vec<_>>(), vec!["st_a", "st_b"]);
}

#[test]
fn text_that_already_matches_is_counted_rather_than_replaced() {
  let (merged, outcome) = TranslationMerger::merge(
    document(r#"{"st_a":{"eng":"A"}}"#),
    &[(String::from("st_a"), String::from("A"))],
    "eng",
    false,
  );

  assert_eq!(outcome.unchanged, 1);
  assert_eq!(
    merged["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("A")))
  );
}
