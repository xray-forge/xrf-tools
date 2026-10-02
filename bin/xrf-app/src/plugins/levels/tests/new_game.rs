use xrf_error::XrfResult;

use crate::plugins::levels::new_game::list_released;

#[test]
fn lists_the_names_a_section_keys_without_values() -> XrfResult {
  let text: &str =
    "; objects to remove\n[remove_objects]\nesc_btr\nesc_zone_mine_field_soc_0000\n\n[other]\nkept = 1\n";

  assert_eq!(
    list_released(text, "remove_objects")?,
    vec!["esc_btr".to_owned(), "esc_zone_mine_field_soc_0000".to_owned()]
  );

  Ok(())
}

#[test]
fn lists_nothing_of_a_missing_section() -> XrfResult {
  assert!(list_released("[settings]\nenabled = true\n", "replace_items")?.is_empty());

  Ok(())
}
