use xrf_error::XrfResult;
use xrf_test_utils::utils::write_generated_test_resource;
use xrf_utils::new_windows1251_encoder;

use crate::dialog::Dialog;
use crate::element::DialogElementKind;
use crate::file::DialogFile;
use crate::phrase::DialogPhrase;

/// Parse decoded text, which is what most of these tests need.
fn parse(source: &str) -> XrfResult<DialogFile> {
  DialogFile::parse(String::from(source), new_windows1251_encoder())
}

#[test]
fn reads_dialogs_in_document_order() -> XrfResult {
  let file: DialogFile = parse(
    r#"<game_dialogs>
      <dialog id="second"/>
      <dialog id="first"/>
    </game_dialogs>"#,
  )?;

  assert_eq!(
    file.get_dialogs().iter().map(Dialog::get_id).collect::<Vec<&str>>(),
    vec!["second", "first"]
  );

  Ok(())
}

#[test]
fn reads_a_dialog_that_declares_no_phrases() -> XrfResult {
  // `dm_traveler_dialog` is shaped like this: it builds its phrases from script at runtime.
  let file: DialogFile = parse(
    r#"<game_dialogs>
      <dialog id="dm_traveler_dialog">
        <precondition>travel_callbacks.can_start_traveling_dialogs</precondition>
        <init_func>travel_callbacks.initialize_traveller_dialog</init_func>
      </dialog>
    </game_dialogs>"#,
  )?;

  let dialog: &Dialog = file
    .find_dialog("dm_traveler_dialog")
    .expect("the dialog should be read");

  assert!(!dialog.has_phrases());
  assert_eq!(
    dialog.list_values_of(DialogElementKind::Precondition),
    vec!["travel_callbacks.can_start_traveling_dialogs"]
  );
  assert_eq!(
    dialog.list_values_of(DialogElementKind::InitFunc),
    vec!["travel_callbacks.initialize_traveller_dialog"]
  );
  assert!(file.get_issues().is_empty());

  Ok(())
}

#[test]
fn reads_phrases_from_a_phrase_list() -> XrfResult {
  let file: DialogFile = parse(
    r#"<game_dialogs>
      <dialog id="about_skadovsk_dialog_stalkers">
        <precondition>dialogs.npc_stalker</precondition>
        <dont_has_info>about_skadovsk_dialog_done</dont_has_info>
        <phrase_list>
          <phrase id="0">
            <text>about_skadovsk_dialog_stalkers_0</text>
            <next>1</next>
          </phrase>
          <phrase id="1">
            <give_info>about_skadovsk_dialog_done</give_info>
            <text>about_skadovsk_dialog_stalkers_1</text>
          </phrase>
        </phrase_list>
      </dialog>
    </game_dialogs>"#,
  )?;

  let dialog: &Dialog = &file.get_dialogs()[0];

  assert_eq!(dialog.get_phrases().len(), 2);
  assert_eq!(file.sum_phrases(), 2);

  let entry: &DialogPhrase = dialog.find_phrase("0").expect("phrase 0 should be read");

  assert!(entry.is_in_phrase_list());
  assert_eq!(entry.get_text(), Some("about_skadovsk_dialog_stalkers_0"));
  assert_eq!(entry.list_next(), vec!["1"]);
  assert!(!entry.is_final());

  Ok(())
}

#[test]
fn reads_a_phrase_written_directly_under_its_dialog() -> XrfResult {
  // `hello_dialog` writes a bare self-closing phrase instead of a list, and it is the only one in the
  // shipped files that does.
  let file: DialogFile = parse(
    r#"<game_dialogs>
      <dialog id="hello_dialog">
        <init_func>dialog_manager.init_hello_dialogs</init_func>
        <phrase id="0"/>
      </dialog>
    </game_dialogs>"#,
  )?;

  let phrase: &DialogPhrase = file
    .find_dialog("hello_dialog")
    .and_then(|dialog| dialog.find_phrase("0"))
    .expect("the bare phrase should be read");

  assert!(!phrase.is_in_phrase_list());
  assert_eq!(phrase.get_text(), None);
  assert!(phrase.get_elements().is_empty());
  assert!(file.get_issues().is_empty());

  Ok(())
}

#[test]
fn reads_a_negative_priority() -> XrfResult {
  let file: DialogFile = parse(r#"<game_dialogs><dialog id="actor_break_dialog" priority="-5"/></game_dialogs>"#)?;

  assert_eq!(file.get_dialogs()[0].get_priority(), Some(-5));

  Ok(())
}

#[test]
fn leaves_priority_absent_when_it_is_not_declared() -> XrfResult {
  let file: DialogFile = parse(r#"<game_dialogs><dialog id="plain"/></game_dialogs>"#)?;

  assert_eq!(file.get_dialogs()[0].get_priority(), None);

  Ok(())
}

#[test]
fn keeps_duplicate_ids_rather_than_refusing_them() -> XrfResult {
  // Which one the engine reaches is a validation rule, not a parsing one. Refusing here would make
  // the editor unable to open the file it exists to fix.
  let file: DialogFile = parse(
    r#"<game_dialogs>
      <dialog id="dup"><phrase_list><phrase id="0"><text>first</text></phrase></phrase_list></dialog>
      <dialog id="dup"><phrase_list><phrase id="0"><text>second</text></phrase></phrase_list></dialog>
    </game_dialogs>"#,
  )?;

  assert_eq!(file.get_dialogs().len(), 2);
  // `find_dialog` answers the first match, which is the one the engine reaches too.
  assert_eq!(
    file
      .find_dialog("dup")
      .and_then(|dialog| dialog.find_phrase("0"))
      .and_then(DialogPhrase::get_text),
    Some("first")
  );

  Ok(())
}

#[test]
fn ranges_address_the_source_it_kept() -> XrfResult {
  let source: &str = r#"<game_dialogs><dialog id="ranged"><phrase_list><phrase id="0"><text>key</text></phrase></phrase_list></dialog></game_dialogs>"#;
  let file: DialogFile = parse(source)?;
  let dialog: &Dialog = &file.get_dialogs()[0];
  let phrase: &DialogPhrase = &dialog.get_phrases()[0];

  assert_eq!(file.get_source(), source);
  assert!(file.get_source()[dialog.get_range().clone()].starts_with("<dialog id=\"ranged\">"));
  assert_eq!(
    &file.get_source()[phrase.get_range().clone()],
    "<phrase id=\"0\"><text>key</text></phrase>"
  );
  assert_eq!(
    &file.get_source()[phrase.get_elements()[0].get_range().clone()],
    "<text>key</text>"
  );

  Ok(())
}

#[test]
fn opens_a_file_declaring_windows_1251() -> XrfResult {
  let path = write_generated_test_resource(
    "dialog_file/declared.xml",
    // `Привет` in windows-1251, which is not valid UTF-8 and proves the declaration was honoured.
    [
      br#"<?xml version="1.0" encoding="windows-1251" ?><game_dialogs><dialog id="greet"><phrase_list><phrase id="0"><text>"#
        .as_slice(),
      &[0xCF, 0xF0, 0xE8, 0xE2, 0xE5, 0xF2],
      b"</text></phrase></phrase_list></dialog></game_dialogs>".as_slice(),
    ]
    .concat(),
  )?;

  let file: DialogFile = DialogFile::read_from_path(&path)?;

  assert_eq!(file.get_encoding().name(), "windows-1251");
  assert_eq!(
    file
      .find_dialog("greet")
      .and_then(|dialog| dialog.find_phrase("0"))
      .and_then(DialogPhrase::get_text),
    Some("Привет")
  );

  Ok(())
}

#[test]
fn refuses_utf16_which_no_config_encoder_can_hold() {
  let bytes: Vec<u8> = [&[0xFF, 0xFE][..], b"<game_dialogs/>"].concat();

  assert!(DialogFile::read_from_bytes(&bytes).is_err());
}
