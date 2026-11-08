use xrf_error::XrfResult;

use crate::{IniField, IniFile, IniSection};

#[test]
fn reads_sections_and_trimmed_fields_skipping_comments_and_blank_lines() -> XrfResult {
  let file: IniFile =
    IniFile::read_from_str("; header\r\n[window]\r\n  width =  800 \r\n\r\nfullscreen\r\n[empty]\r\n")?;

  assert_eq!(
    file.sections,
    vec![
      IniSection {
        name: "window".into(),
        fields: vec![IniField::new("width", "800"), IniField::new("fullscreen", "")],
      },
      IniSection::new("empty"),
    ]
  );

  Ok(())
}

#[test]
fn writes_fields_in_the_order_given_with_no_padding() -> XrfResult {
  let mut file: IniFile = IniFile::default();

  file.section_mut("b").set("z", "1");
  file.section_mut("b").set("a", "");
  file.section_mut("a").set("key", "value");
  file.section_mut("b").set("z", "2");

  assert_eq!(file.write_to_string()?, "[b]\nz = 2\na =\n[a]\nkey = value\n");

  Ok(())
}

#[test]
fn a_plain_file_comes_back_as_it_was_written() -> XrfResult {
  let text: &str = "[a]\nx = 1, 2\nquoted = \"a; b = c\"\n[b]\n";

  assert_eq!(IniFile::read_from_str(text)?.write_to_string()?, text);
  assert_eq!(
    IniFile::read_from_bytes(text.as_bytes())?.write_to_bytes()?,
    text.as_bytes()
  );

  Ok(())
}

#[test]
fn a_plain_read_keeps_an_open_quote_to_its_line() -> XrfResult {
  let file: IniFile = IniFile::read_from_str("[a]\nx = \"open\ny = 1\n")?;

  assert_eq!(file.get("a", "x"), Some("\"open"));
  assert_eq!(file.get("a", "y"), Some("1"));

  Ok(())
}

#[test]
fn parses_a_field_whole() -> XrfResult {
  let file: IniFile = IniFile::read_from_str("[a]\nn = 42\nf = 4x\n")?;
  let section: &IniSection = file.section_required("a")?;

  assert_eq!(section.get_parsed::<u32>("n")?, 42);
  assert!(section.get_parsed::<u32>("f").is_err());
  assert!(section.get_parsed::<u32>("missing").is_err());
  assert!(file.section_required("missing").is_err());

  Ok(())
}

#[test]
fn removes_a_field() -> XrfResult {
  let mut file: IniFile = IniFile::read_from_str("[a]\nx = 1\ny = 2\n")?;

  assert_eq!(file.section_mut("a").remove("x"), Some("1".into()));
  assert_eq!(file.section_mut("a").remove("x"), None);
  assert_eq!(file.write_to_string()?, "[a]\ny = 2\n");

  Ok(())
}

#[test]
fn refuses_text_that_is_not_one_file_of_sections() {
  for (text, reason) in [
    ("x = 1\n", "a field before any section"),
    ("[a]\nx = 1\nx = 2\n", "a key set twice"),
    ("[a]\n[a]\n", "a section declared twice"),
    ("[a]:base\n", "a section with parents"),
    ("[a\n", "a header with no closing bracket"),
    ("[a]\n= 1\n", "a field with no key"),
  ] {
    assert!(IniFile::read_from_str(text).is_err(), "{reason} was read");
  }
}

#[test]
fn refuses_to_write_what_would_not_read_back() {
  for (section, field, reason) in [
    ("", IniField::new("k", "v"), "an empty section name"),
    ("a]", IniField::new("k", "v"), "a bracket in a section name"),
    ("a", IniField::new("", "v"), "an empty key"),
    ("a", IniField::new("k=", "v"), "a key holding '='"),
    ("a", IniField::new(" k", "v"), "a padded key"),
    ("a", IniField::new(";k", "v"), "a key read as a comment"),
    ("a", IniField::new("k", " v"), "a padded value"),
    ("a", IniField::new("k", "\"a\nb\""), "a line break in a plain value"),
  ] {
    let file: IniFile = IniFile {
      sections: vec![IniSection {
        name: section.into(),
        fields: vec![field],
      }],
      ..IniFile::default()
    };

    assert!(file.write_to_string().is_err(), "{reason} was written");
  }
}

#[test]
fn refuses_bytes_that_are_not_utf8() {
  assert!(IniFile::read_from_bytes(&[b'[', 0xFF, b']']).is_err());
}
