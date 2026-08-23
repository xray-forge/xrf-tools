use xrf_error::XrfResult;
use xrf_utils::new_windows1251_encoder;

use crate::element::DialogElementKind;
use crate::file::DialogFile;
use crate::phrase::DialogPhrase;

/// Parse one dialog holding one phrase with the given body.
fn phrase_of(body: &str) -> XrfResult<DialogPhrase> {
  let file: DialogFile = DialogFile::parse(
    format!(
      r#"<game_dialogs><dialog id="d"><phrase_list><phrase id="0">{body}</phrase></phrase_list></dialog></game_dialogs>"#
    ),
    new_windows1251_encoder(),
  )?;

  Ok(file.get_dialogs()[0].get_phrases()[0].clone())
}

#[test]
fn keeps_next_in_file_order() -> XrfResult {
  // File order is the order the player is offered the options, so this sequence is game behavior.
  let phrase: DialogPhrase = phrase_of("<next>actor_I_wanna_eat_smth</next><next>actor_ok_thx_for_welcome</next>")?;

  assert_eq!(
    phrase.list_next(),
    vec!["actor_I_wanna_eat_smth", "actor_ok_thx_for_welcome"]
  );

  Ok(())
}

#[test]
fn keeps_repeated_elements_of_one_kind() -> XrfResult {
  // 937 `give_info` elements across the shipped files, and a phrase may carry several.
  let phrase: DialogPhrase = phrase_of("<give_info>one</give_info><text>key</text><give_info>two</give_info>")?;

  assert_eq!(phrase.list_values_of(DialogElementKind::GiveInfo), vec!["one", "two"]);
  assert_eq!(phrase.get_text(), Some("key"));

  Ok(())
}

#[test]
fn keeps_elements_in_document_order_across_kinds() -> XrfResult {
  let phrase: DialogPhrase = phrase_of("<give_info>i</give_info><text>t</text><next>1</next>")?;

  assert_eq!(
    phrase
      .get_elements()
      .iter()
      .map(|element| element.get_name())
      .collect::<Vec<&str>>(),
    vec!["give_info", "text", "next"]
  );

  Ok(())
}

#[test]
fn reports_a_final_phrase() -> XrfResult {
  assert!(phrase_of("<text>bye</text><is_final>1</is_final>")?.is_final());
  assert!(!phrase_of("<text>bye</text>")?.is_final());

  Ok(())
}

#[test]
fn answers_which_elements_a_phrase_carries() -> XrfResult {
  let phrase: DialogPhrase = phrase_of("<text>key</text><precondition>dialogs.npc_stalker</precondition>")?;

  assert!(phrase.has_element_of(DialogElementKind::Precondition));
  assert!(!phrase.has_element_of(DialogElementKind::Action));
  assert!(phrase.list_values_of(DialogElementKind::Action).is_empty());

  Ok(())
}

#[test]
fn resolves_entities_in_a_value() -> XrfResult {
  let phrase: DialogPhrase = phrase_of("<text>a &amp; b</text>")?;

  assert_eq!(phrase.get_text(), Some("a & b"));

  Ok(())
}

#[test]
fn reads_a_script_text_phrase_with_no_translation_key() -> XrfResult {
  // Two phrases in the shipped files build their line from script instead of a key.
  let phrase: DialogPhrase = phrase_of("<script_text>dialog_manager.create_bye_phrase</script_text>")?;

  assert_eq!(phrase.get_text(), None);
  assert_eq!(
    phrase.list_values_of(DialogElementKind::ScriptText),
    vec!["dialog_manager.create_bye_phrase"]
  );

  Ok(())
}
