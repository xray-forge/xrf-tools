use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;
use xrf_utils::encode_string_to_w1251_bytes;

use crate::{IniEncoding, IniFile, IniFormat, IniOverride};

/// A file shaped as the SDK's `CInifile::save_as` writes one: a long key past the column, a key with no value, a
/// decorated vector, Cyrillic text, and a spawn object's custom data spanning lines with `;`, `=` and brackets inside.
const SAVED: &str = concat!(
  "[guid]\r\n",
  "        guid_g0                          = 5065284203709945560\r\n",
  " \r\n",
  "[object_12_spawndata]\r\n",
  "        000024                           = \"[logic]\r\n",
  "cfg = scripts\\labx8\\a.ltx ; kept\r\n",
  "\r\n",
  "[story_object]\r\n",
  "story_id = lift\"\r\n",
  "        000025                           = -1\r\n",
  "        a_key_longer_than_the_thirty_two_columns = 1\r\n",
  "        name                             =\r\n",
  "        owner                            = Кордон\r\n",
  "        position                         = -43.298708, -20.553184, 67.121576\r\n",
  " \r\n",
);

#[test]
fn a_saved_file_comes_back_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = encode_string_to_w1251_bytes(SAVED)?;
  let file: IniFile = IniFile::read_from_bytes_as(&bytes, &IniFormat::XRAY_SAVE_AS)?;

  assert_eq!(
    file.get("object_12_spawndata", "000024"),
    Some("\"[logic]\r\ncfg = scripts\\labx8\\a.ltx ; kept\r\n\r\n[story_object]\r\nstory_id = lift\"")
  );
  assert_eq!(file.get("object_12_spawndata", "name"), Some(""));
  assert_eq!(file.get("object_12_spawndata", "owner"), Some("Кордон"));
  assert_eq!(file.write_to_bytes_as(&IniFormat::XRAY_SAVE_AS)?, bytes);

  Ok(())
}

#[test]
fn writes_sections_and_fields_in_strcmp_order() -> XrfResult {
  let mut file: IniFile = IniFile::default();

  file.section_mut("object_2").set("b", "1");
  file.section_mut("object_10").set("z", "1");
  file.section_mut("object_10").set("a", "1");
  file.section_mut("object_1_attached_0").set("k", "1");

  let text: String = file.write_to_string_as(&IniFormat::XRAY_SAVE_AS)?;
  let names: Vec<&str> = text.lines().filter(|line| line.starts_with('[')).collect();

  assert_eq!(names, vec!["[object_10]", "[object_1_attached_0]", "[object_2]"]);
  assert!(text.find("        a ").unwrap() < text.find("        z ").unwrap());

  Ok(())
}

#[test]
fn orders_windows_1251_names_by_their_bytes() {
  // «№» is 0xB9 in Windows-1251 and «А» 0xC0, so `strcmp` puts «№» first; as code points (U+2116, U+0410) the order
  // is the other way round.
  assert!(IniEncoding::Windows1251.compare("№", "А").is_lt());
  assert!(IniEncoding::Utf8.compare("№", "А").is_gt());
}

#[test]
fn a_key_set_twice_keeps_its_last_value_and_names_the_dropped_one() -> XrfResult {
  // bardak's converter writes a glow's `flags` twice; `CInifile::Load` keeps the second.
  let text: &str = "[object_0]\r\n flags = 1\r\n name = glow\r\n flags = 0\r\n";
  let file: IniFile = IniFile::read_from_str_as(text, &IniFormat::XRAY_SAVE_AS)?;

  assert_eq!(file.get("object_0", "flags"), Some("0"));
  assert_eq!(
    file.overridden,
    vec![IniOverride {
      section: "object_0".into(),
      key: "flags".into(),
      value: "1".into(),
      line: 4,
    }]
  );
  assert_eq!(
    file
      .write_to_string_as(&IniFormat::XRAY_SAVE_AS)?
      .matches("flags")
      .count(),
    1
  );
  assert!(IniFile::read_from_str(text).is_err(), "a plain read refuses the repeat");

  Ok(())
}

#[test]
fn refuses_a_quote_left_open_at_the_end() {
  let text: &str = "[a]\r\n        x                                = \"open\r\nstill\r\n";

  assert!(IniFile::read_from_str_as(text, &IniFormat::XRAY_SAVE_AS).is_err());
}

#[test]
fn refuses_a_line_break_outside_a_quoted_string() {
  let mut file: IniFile = IniFile::default();

  file.section_mut("a").set("x", "a\r\nb");

  assert!(file.write_to_string_as(&IniFormat::XRAY_SAVE_AS).is_err());
}

#[test]
fn a_file_written_to_a_path_reads_back() -> XrfResult {
  let path = GeneratedResources::path("xrf-ini/xray/saved.ltx");
  let file: IniFile = IniFile::read_from_str_as(SAVED, &IniFormat::XRAY_SAVE_AS)?;

  file.write_to_path_as(&path, &IniFormat::XRAY_SAVE_AS)?;

  assert_eq!(IniFile::read_from_path_as(&path, &IniFormat::XRAY_SAVE_AS)?, file);

  Ok(())
}
