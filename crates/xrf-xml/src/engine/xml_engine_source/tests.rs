use std::collections::HashMap;

use xrf_error::{XrfError, XrfResult};

use crate::engine::XmlEngineSource;
use crate::source::XmlSourceDocument;

/// Read `source` as the engine does against a set of files keyed by logical path.
fn parse(source: &[u8], files: &[(&str, &[u8])]) -> XrfResult<XmlSourceDocument> {
  let files: HashMap<&str, &[u8]> = files.iter().copied().collect();

  XmlEngineSource::parse(source, &mut |path: &str| {
    files
      .get(path)
      .map(|content| content.to_vec())
      .ok_or_else(|| XrfError::new_not_found_error(format!("No file '{path}'")))
  })
}

#[test]
fn reads_a_document_through_its_parts_past_a_comment_banner() -> XrfResult {
  // A part has no root of its own, and the banner's `--` is what the strict reader refuses.
  let document: XmlSourceDocument = parse(
    b"<?xml version=\"1.0\" encoding=\"windows-1251\"?>\r\n<!-- -- characters -- -->\r\n<characters>\r\n\
      #include \"gameplay\\character_dialogs.xml\"\r\n</characters>\r\n",
    &[(
      "configs\\gameplay\\character_dialogs.xml",
      b"<?xml version=\"1.0\" encoding=\"windows-1251\"?>\r\n<sound>weapons\\shot</sound>\r\n<sound>ambient\\wind</sound>",
    )],
  )?;

  let sounds: Vec<&str> = document.elements_named("sound").map(|sound| sound.text()).collect();

  assert_eq!(sounds, ["weapons\\shot", "ambient\\wind"]);

  Ok(())
}

#[test]
fn names_the_parts_a_tree_s_documents_include() {
  let first: &[u8] = b"<a>\n#include \"gameplay\\part.xml\"\n</a>";
  let second: &[u8] = b"<b/>";

  assert_eq!(
    XmlEngineSource::list_included([first, second]),
    ["configs\\gameplay\\part.xml"]
  );
}

#[test]
fn reports_a_part_the_tree_does_not_hold() {
  let error: String = parse(b"<a>\n#include \"missing.xml\"\n</a>", &[])
    .unwrap_err()
    .to_string();

  assert!(error.contains("No file 'configs\\missing.xml'"), "{error}");
}

#[test]
fn decodes_windows_1251_unless_a_mark_or_declaration_says_otherwise() -> XrfResult {
  // `Зона` in windows-1251, then in UTF-8 under a byte order mark that outranks the declaration.
  assert_eq!(XmlEngineSource::decode(b"<a>\xC7\xEE\xED\xE0</a>")?, "<a>Зона</a>");
  assert_eq!(
    XmlEngineSource::decode("\u{FEFF}<?xml version=\"1.0\" encoding=\"windows-1251\"?><a>Зона</a>".as_bytes())?,
    "<?xml version=\"1.0\" encoding=\"windows-1251\"?><a>Зона</a>"
  );
  assert_eq!(
    XmlEngineSource::decode("<?xml version=\"1.0\" encoding=\"utf-8\"?><a>Зона</a>".as_bytes())?,
    "<?xml version=\"1.0\" encoding=\"utf-8\"?><a>Зона</a>"
  );

  Ok(())
}

#[test]
fn refuses_utf16() {
  let error: String = XmlEngineSource::decode(&[0xFF, 0xFE, b'<', 0, b'a', 0, b'/', 0, b'>', 0])
    .unwrap_err()
    .to_string();

  assert!(error.contains("UTF-16"), "{error}");
}

#[test]
fn splits_a_byte_order_mark_off_for_a_rewrite() {
  assert_eq!(
    XmlEngineSource::split_byte_order_mark(b"\xEF\xBB\xBF<a/>"),
    (&b"\xEF\xBB\xBF"[..], &b"<a/>"[..])
  );
  assert_eq!(
    XmlEngineSource::split_byte_order_mark(b"<a/>"),
    (&b""[..], &b"<a/>"[..])
  );
}
