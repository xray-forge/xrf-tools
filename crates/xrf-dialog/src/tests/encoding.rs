use xrf_error::XrfResult;
use xrf_test_utils::utils::write_generated_test_resource;

use crate::encoding::{decode, resolve_encoding};
use crate::file::DialogFile;

#[test]
fn falls_back_to_windows_1251_when_nothing_is_declared() -> XrfResult {
  // What the engine assumes for gameplay configs.
  assert_eq!(resolve_encoding(b"<game_dialogs/>")?.name(), "windows-1251");

  Ok(())
}

#[test]
fn honours_a_declared_encoding() -> XrfResult {
  assert_eq!(
    resolve_encoding(br#"<?xml version="1.0" encoding="utf-8" ?><game_dialogs/>"#)?.name(),
    "UTF-8"
  );

  Ok(())
}

#[test]
fn decodes_windows_1251_text_that_is_not_valid_utf8() -> XrfResult {
  let bytes: Vec<u8> = [
    br#"<game_dialogs><dialog id="d"><phrase_list><phrase id="0"><text>"#.as_slice(),
    // `Зона` in windows-1251.
    &[0xC7, 0xEE, 0xED, 0xE0],
    b"</text></phrase></phrase_list></dialog></game_dialogs>".as_slice(),
  ]
  .concat();

  assert!(String::from_utf8(bytes.clone()).is_err());

  let file: DialogFile = DialogFile::read_from_bytes(&bytes)?;

  assert_eq!(file.get_dialogs()[0].get_phrases()[0].get_text(), Some("Зона"));
  assert_eq!(file.get_encoding().name(), "windows-1251");
  assert!(file.get_byte_order_mark().is_empty());

  Ok(())
}

#[test]
fn keeps_a_byte_order_mark_out_of_the_text_and_holds_it_for_a_rewrite() -> XrfResult {
  // A mark outranks the declaration, which shipped files contradict. Kept verbatim, because the
  // usual decode strips it and re-encoding would then drop it from a file that had one.
  let bytes: Vec<u8> = [
    &[0xEF, 0xBB, 0xBF][..],
    br#"<?xml version="1.0" encoding="windows-1251" ?><game_dialogs><dialog id="marked"/></game_dialogs>"#,
  ]
  .concat();

  let decoded = decode(&bytes)?;

  assert_eq!(decoded.byte_order_mark, vec![0xEF, 0xBB, 0xBF]);
  assert_eq!(decoded.encoding.name(), "UTF-8");
  assert!(decoded.text.starts_with("<?xml"));

  let file: DialogFile = DialogFile::read_from_bytes(&bytes)?;

  assert_eq!(file.get_byte_order_mark(), &[0xEF, 0xBB, 0xBF]);
  assert!(file.find_dialog("marked").is_some());

  Ok(())
}

#[test]
fn reads_the_same_bytes_from_a_path_and_from_memory() -> XrfResult {
  let source: &[u8] =
    br#"<?xml version="1.0" encoding="windows-1251" ?><game_dialogs><dialog id="both"/></game_dialogs>"#;
  let path = write_generated_test_resource("dialog_encoding/both.xml", source)?;

  let from_path: DialogFile = DialogFile::read_from_path(&path)?;
  let from_bytes: DialogFile = DialogFile::read_from_bytes(source)?;

  assert_eq!(from_path.get_source(), from_bytes.get_source());
  assert_eq!(from_path.get_encoding().name(), from_bytes.get_encoding().name());

  Ok(())
}
