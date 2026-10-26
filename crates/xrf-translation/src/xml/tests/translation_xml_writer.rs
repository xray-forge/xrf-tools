use std::fs;
use std::path::Path;

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;
use xrf_utils::{encode_string_to_bytes, new_windows1251_encoder, new_windows1252_encoder};

use crate::text::translation_edit::TranslationEdit;
use crate::text::translation_variant::TranslationVariant;
use crate::xml::translation_xml_writer::TranslationXmlWriter;

/// Shaped like shipped gamedata: CRLF, tabs, a section comment, and a duplicated id.
const SOURCE: &str = concat!(
  "<?xml version=\"1.0\" encoding=\"windows-1251\" ?>\r\n",
  "<string_table>\r\n",
  "\r\n",
  "\t<!-- UI STRINGS -->\r\n",
  "\r\n",
  "\t<string id=\"st_first\">\r\n",
  "\t\t<text>first</text>\r\n",
  "\t</string>\r\n",
  "\t<string id=\"st_dup\">\r\n",
  "\t\t<text>shadowed</text>\r\n",
  "\t</string>\r\n",
  "\t<string id=\"st_dup\">\r\n",
  "\t\t<text>winning</text>\r\n",
  "\t</string>\r\n",
  "</string_table>\r\n",
);

fn splice(edits: &[TranslationEdit]) -> String {
  TranslationXmlWriter::splice(
    Path::new("st_test.xml"),
    String::from(SOURCE),
    edits,
    new_windows1252_encoder(),
  )
  .expect("Expected a spliced document")
}

fn set(id: &str, text: &str) -> TranslationEdit {
  TranslationEdit::Set {
    id: String::from(id),
    value: TranslationVariant::String(String::from(text)),
  }
}

#[test]
fn an_edit_changes_nothing_but_the_edited_value() {
  let edited: String = splice(&[set("st_first", "replaced")]);

  assert_eq!(edited, SOURCE.replace("<text>first</text>", "<text>replaced</text>"));
  // Spelled out because it is the whole point: the comment, the blank lines, the tabs and the CRLF
  // endings are all still there, and a re-serialize would have dropped every one of them.
  assert!(edited.contains("\t<!-- UI STRINGS -->\r\n"));
  assert!(edited.contains("<?xml version=\"1.0\" encoding=\"windows-1251\" ?>"));
}

#[test]
fn an_edit_lands_on_the_occurrence_the_engine_resolves() {
  let edited: String = splice(&[set("st_dup", "corrected")]);

  // Last wins in CStringTable::Load, so editing the shadowed one would appear to do nothing in game.
  assert!(edited.contains("<text>shadowed</text>"));
  assert!(edited.contains("<text>corrected</text>"));
  assert!(!edited.contains("<text>winning</text>"));
}

#[test]
fn removing_an_id_takes_its_shadowed_duplicates_with_it() {
  let edited: String = splice(&[TranslationEdit::Remove {
    id: String::from("st_dup"),
  }]);

  assert!(!edited.contains("st_dup"));
  assert!(!edited.contains("shadowed"));
  // No indented blank left where the entries were.
  assert!(edited.contains("\t</string>\r\n</string_table>"));
}

#[test]
fn a_new_entry_copies_the_indentation_and_line_endings_it_lands_among() {
  let edited: String = splice(&[set("st_added", "added")]);

  assert!(edited.contains("\t<string id=\"st_added\">\r\n\t\t<text>added</text>\r\n\t</string>\r\n</string_table>"));
}

#[test]
fn several_edits_apply_without_shifting_each_other() {
  let edited: String = splice(&[set("st_first", "one"), set("st_dup", "two"), set("st_new", "three")]);

  assert!(edited.contains("<text>one</text>"));
  assert!(edited.contains("<text>two</text>"));
  assert!(edited.contains("<string id=\"st_new\">"));
}

#[test]
fn reserved_characters_are_escaped_on_the_way_in() {
  let edited: String = splice(&[set("st_first", "a & b < c")]);

  assert!(edited.contains("<text>a &amp; b &lt; c</text>"));
}

#[test]
fn a_value_the_encoding_cannot_hold_is_refused_with_the_character_named() {
  let error = TranslationXmlWriter::splice(
    Path::new("st_test.xml"),
    String::from(SOURCE),
    &[set("st_first", "Привет")],
    new_windows1252_encoder(),
  )
  .unwrap_err();

  assert!(error.to_string().contains("cannot be encoded as windows-1252"));
  assert!(error.to_string().contains("U+041F"));
}

#[test]
fn writing_a_file_leaves_every_untouched_byte_alone() -> XrfResult {
  let relative_path: &str = "xml_write/in_place.rus.xml";
  let path = GeneratedResources::write(
    relative_path,
    encode_string_to_bytes(SOURCE, new_windows1251_encoder())?,
  )?;

  TranslationXmlWriter::apply_edits(&path, &[set("st_first", "Привет")])?;

  let expected: Vec<u8> = encode_string_to_bytes(
    &SOURCE.replace("<text>first</text>", "<text>Привет</text>"),
    new_windows1251_encoder(),
  )?;

  assert_eq!(fs::read(&path)?, expected);

  Ok(())
}

#[test]
fn a_byte_order_mark_survives_a_rewrite() -> XrfResult {
  let relative_path: &str = "xml_write/marked.xml";
  let body: &str = "<string_table><string id=\"st_a\"><text>A</text></string></string_table>";
  let mut marked: Vec<u8> = vec![0xEF, 0xBB, 0xBF];

  marked.extend_from_slice(body.as_bytes());

  let path = GeneratedResources::write(relative_path, marked)?;

  TranslationXmlWriter::apply_edits(&path, &[set("st_a", "B")])?;

  let written: Vec<u8> = fs::read(&path)?;

  // Dropping it would change the first three bytes of a file the edit never asked to touch.
  assert!(written.starts_with(&[0xEF, 0xBB, 0xBF]));
  assert!(String::from_utf8_lossy(&written).contains("<text>B</text>"));

  Ok(())
}

#[test]
fn an_edit_that_would_leave_a_line_the_game_refuses_is_refused() {
  let long: String = "a".repeat(5000);
  let error =
    TranslationXmlWriter::render_edits(Path::new("st_test.xml"), SOURCE.as_bytes(), &[set("st_first", &long)])
      .unwrap_err();

  assert!(error.to_string().contains("would not load"));
  assert!(error.to_string().contains("line 7"));
}

#[test]
fn a_long_line_the_file_already_held_does_not_block_another_edit() -> XrfResult {
  // The table never loaded with it, and a save that does not touch it makes nothing worse.
  let source: String = SOURCE.replace("<text>winning</text>", &format!("<text>{}</text>", "w".repeat(5000)));
  let written: Vec<u8> = TranslationXmlWriter::render_edits(
    Path::new("st_test.xml"),
    source.as_bytes(),
    &[set("st_first", "replaced")],
  )?;

  assert!(String::from_utf8_lossy(&written).contains("<text>replaced</text>"));

  Ok(())
}

#[test]
fn an_empty_edit_list_does_not_touch_the_file() -> XrfResult {
  let relative_path: &str = "xml_write/untouched.xml";
  let body: &str = "<string_table><string id=\"st_a\"><text>A</text></string></string_table>";
  let path = GeneratedResources::write(relative_path, body)?;

  TranslationXmlWriter::apply_edits(&path, &[])?;

  assert_eq!(fs::read(&path)?, body.as_bytes());

  Ok(())
}
